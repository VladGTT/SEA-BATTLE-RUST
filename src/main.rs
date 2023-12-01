mod battle_prep_window;
mod battle_results_window;
mod battle_window;
mod connection;
mod draw_table;
mod play_field;
mod stats;

use fltk::{
    app::{self, Receiver as AppReciever, Sender as AppSender},
    enums,
    prelude::*,
    window,
};
use std::sync::mpsc::{Receiver, Sender};

use stats::{BattleResults, BattleStatistics};

use connection::{Connection, Message};
use play_field::{Field, GameField, PlayField, PrepField, StrikeResponce};

use battle_prep_window::{BattlePrepWindow, BattlePreparationEvents};
use battle_results_window::*;
use battle_window::{BattleWindow, BattleWindowEvents};

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET: &str = "localhost:8888";

enum GUIEvents {
    RedrawBattleWindow(PlayField, PlayField),
    RedrawPreparationsWindow(PlayField),
    RedrawResultsWindow(BattleStatistics),

    DisableBattleWindow,
    EnableBattleWindow,

    HideBattleWindow,
    ShowBattleWindow,

    HidePreparationsWindow,
    ShowPreparationsWindow,

    HideResultsWindow,
    ShowResultsWindow,
}

fn prepare_field(s: AppSender<GUIEvents>, r: &Receiver<BattlePreparationEvents>) -> PlayField {
    let mut player_field = PlayField::new_player_field();

    s.send(GUIEvents::RedrawPreparationsWindow(player_field));

    loop {
        std::thread::sleep(std::time::Duration::from_millis(5));
        if let Ok(msg) = r.recv() {
            match msg {
                BattlePreparationEvents::Ready => {
                    if player_field.get_ship_numb() != (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK)
                    {
                        continue;
                    }
                    s.send(GUIEvents::RedrawPreparationsWindow(player_field));
                    break;
                }
                BattlePreparationEvents::Reset => {
                    player_field.reset();
                    s.send(GUIEvents::RedrawPreparationsWindow(player_field));
                }
                BattlePreparationEvents::ShipPlaced(coords) => {
                    player_field.place_ship(coords);
                    s.send(GUIEvents::RedrawPreparationsWindow(player_field));
                }
            }
        }
    }
    player_field
}

fn listen_player_strike(r: &Receiver<BattleWindowEvents>) -> (i32, i32) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        if let Ok(msg) = r.recv() {
            if let BattleWindowEvents::Strike(coords) = msg {
                return coords;
            }
        }
    }
}
fn show_battle_results(r: &Receiver<BattleResultsEvents>) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        if let Ok(msg) = r.recv() {
            if let BattleResultsEvents::ButtonPressed = msg {
                return;
            }
        }
    }
}

// mut res: BattleResultWindow,
async fn handle_game(
    bat_num: usize,
    is_server: bool,
    sender: AppSender<GUIEvents>,
    prep_reciever: Receiver<BattlePreparationEvents>,
    bat_reciever: Receiver<BattleWindowEvents>,
    res_reciever: Receiver<BattleResultsEvents>,
    conn: Connection,
) {
    let mut my_move = !is_server;
    for _ in 0..bat_num {
        my_move = !my_move;
        
        // Prepare player field
        // res.hide();
        
        sender.send(GUIEvents::ShowPreparationsWindow);
        let result = conn.listen_for(&Message { data: [0, 0] });
        
        let mut player_field = prepare_field(sender.clone(), &prep_reciever);
        
        sender.send(GUIEvents::HidePreparationsWindow);
        
        conn.write(Message { data: [0, 0] });
        
        match result.await {
            Ok(_) => {
                println!("Opponent_is_ready")
            }
            Err(_) => {
                println!("Opponent_is_not_ready");
                return;
            }
        }
        
        // Play battle
        let mut opponent_field = PlayField::new_opponent_field();
        
        sender.send(GUIEvents::RedrawBattleWindow(player_field, opponent_field));
        
        sender.send(GUIEvents::ShowBattleWindow);

        let mut stats = BattleStatistics {
            player_shots_hit: 0,
            player_shots_fired: 0,
            opponent_shots_hit: 0,
            opponent_shots_fired: 0,
            player_ships_destroed: (0, 0, 0, 0),
            opponent_ships_destroed: (0, 0, 0, 0),
            results: None,
        };
        loop {
            if stats.results.is_some() {
                if let BattleResults::PlayerWon = stats.results.as_ref().unwrap() {
                    println!("You won");
                } else {
                    println!("You lost");
                }
                break;
            }
            if my_move {
                let coords = listen_player_strike(&bat_reciever);

                sender.send(GUIEvents::DisableBattleWindow);

                conn.write(Message {
                    data: [coords.0 as u8, coords.1 as u8],
                });

                stats.player_shots_fired += 1;

                match conn.listen().await {
                    None => {
                        println!("Connection_broken");
                    }
                    Some(data) => match data.data {
                        [255, 255] => {
                            opponent_field.mark_as_kill(
                                opponent_field
                                    .check_if_killed((coords.0 as u8, coords.1 as u8))
                                    .unwrap(),
                            );
                            stats.player_shots_hit += 1;
                        }
                        [254, 254] => {
                            opponent_field.mark_as_hit((coords.0 as u8, coords.1 as u8));
                            stats.player_shots_hit += 1;
                        }
                        [253, 253] => {
                            opponent_field.mark_as_miss((coords.0 as u8, coords.1 as u8));
                            my_move = false;
                        }
                        [252, 252] => {
                            opponent_field.mark_as_kill(
                                opponent_field
                                    .check_if_killed((coords.0 as u8, coords.1 as u8))
                                    .unwrap(),
                            );
                            stats.player_shots_hit += 1;

                            stats.results = Some(BattleResults::PlayerWon);
                        }
                        _ => (),
                    },
                }

                sender.send(GUIEvents::EnableBattleWindow);
            } else {
                sender.send(GUIEvents::DisableBattleWindow);

                stats.opponent_shots_fired += 1;
                match conn.listen().await {
                    None => {
                        println!("Connection_broken");
                    }
                    Some(data) => match player_field.strike_coords((data.data[0], data.data[1])) {
                        StrikeResponce::Hit => {
                            conn.write(Message { data: [254, 254] });
                            stats.opponent_shots_hit += 1;
                        }
                        StrikeResponce::Kill => {
                            conn.write(Message { data: [255, 255] });
                            stats.opponent_shots_hit += 1;
                        }
                        StrikeResponce::Miss => {
                            conn.write(Message { data: [253, 253] });
                            my_move = true;
                        }
                        StrikeResponce::KilledLast => {
                            conn.write(Message { data: [252, 252] });
                            stats.opponent_shots_hit += 1;
                            stats.results = Some(BattleResults::PlayerLost);
                        }
                    },
                }
                sender.send(GUIEvents::EnableBattleWindow);
            }
            sender.send(GUIEvents::RedrawBattleWindow(player_field, opponent_field));
        }
        sender.send(GUIEvents::HideBattleWindow);

        sender.send(GUIEvents::RedrawResultsWindow(stats));
        sender.send(GUIEvents::ShowResultsWindow);
        show_battle_results(&res_reciever);
        sender.send(GUIEvents::HideResultsWindow);
        
    }
}

#[tokio::main]
async fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = window::Window::default().with_size(800, 600);

    let (sender, reciever): (AppSender<GUIEvents>, AppReciever<GUIEvents>) = app::channel();
    let (battle_prep_sender, battle_prep_reciever): (
        Sender<BattlePreparationEvents>,
        Receiver<BattlePreparationEvents>,
    ) = std::sync::mpsc::channel();
    let (battle_window_sender, battle_window_reciever): (
        Sender<BattleWindowEvents>,
        Receiver<BattleWindowEvents>,
    ) = std::sync::mpsc::channel();
    let (result_window_sender, result_window_reciever): (
        Sender<BattleResultsEvents>,
        Receiver<BattleResultsEvents>,
    ) = std::sync::mpsc::channel();
    
    
    
    
    let mut battle_window = BattleWindow::new();
    battle_window.set_handler(battle_window_sender);
    battle_window.hide();
    wind.add(&battle_window.group);
    
    let mut battle_prep_window = BattlePrepWindow::new();
    battle_prep_window.set_handler(battle_prep_sender);
    
    wind.add(&battle_prep_window.group);

    let mut battle_results_window = BattleResultWindow::new();
    battle_results_window.set_handler(result_window_sender);
    battle_results_window.hide();

    wind.add(&battle_results_window.group);


    wind.make_resizable(true);

    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            app.quit();
            return;
        }
    });

    let mut mode = String::new();

    let stdin = std::io::stdin();
    
    println!("Write mode");
    
    stdin.read_line(&mut mode).unwrap();
    

    
    let mode_str = mode.trim_end();
    
    let mut battle_num= String::new();
    
    let mut connection: Option<Connection> = None;
    if mode_str == "server" {
        connection.insert(Connection::connect_as_server(SOCKET).unwrap());
        wind.set_label("SEA-BATTLE-SERVER");


        println!("Write battle number");
        stdin.read_line(&mut battle_num).unwrap();
        connection.as_ref().unwrap().write(Message { data: [0,battle_num.trim_end().parse::<u8>().unwrap()]});

    } else {
        connection.insert(Connection::connect_as_client(SOCKET).unwrap());
        wind.set_label("SEA-BATTLE-CLIENT");

        let dat= connection.as_ref().unwrap().listen().await.unwrap().data;
        match dat{
            [0,_]=>{
                battle_num.push_str(&format!("{}",dat[1]))
            }
            _=>()    
        };
    };


    let handle = tokio::spawn(handle_game(
        battle_num.trim_end().parse::<usize>().unwrap(),
        mode_str == "server",
        sender,
        battle_prep_reciever,
        battle_window_reciever,
        result_window_reciever,
        connection.unwrap().clone(),
    ));
    wind.show();
   

    while app.wait() {
        if let Some(msg) = reciever.recv() {
            match msg {
                GUIEvents::HideBattleWindow => {
                    battle_window.hide();
                },
                GUIEvents::ShowBattleWindow => {
                    battle_window.show();
                },

                GUIEvents::HidePreparationsWindow => {
                    battle_prep_window.hide();
                },
                GUIEvents::ShowPreparationsWindow =>{
                    battle_prep_window.show();
                },

                GUIEvents::HideResultsWindow => {
                    battle_results_window.hide();
                }
                GUIEvents::ShowResultsWindow => {
                    battle_results_window.show();
                }

                GUIEvents::DisableBattleWindow => {
                    battle_window.opponent_field.deactivate();
                },
                GUIEvents::EnableBattleWindow => {
                    battle_window.opponent_field.activate();
                },

                GUIEvents::RedrawBattleWindow(player_field, opponent_field) => {
                    battle_window.draw(&player_field, &opponent_field);
                }

                GUIEvents::RedrawPreparationsWindow(field) => {
                    battle_prep_window.draw(&field);
                }

                GUIEvents::RedrawResultsWindow(table) => {
                    battle_results_window.draw(&table)
                }
            }
        }
    }


    app.run().unwrap();
}
