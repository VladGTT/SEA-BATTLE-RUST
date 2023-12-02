mod battle_prep_window;
mod battle_results_window;
mod battle_window;
mod connection;
mod draw_table;
mod play_field;
mod stats;
mod connection_window;

use connection_window::{ConnectionOptions,ConnectionWindow};

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

    ShowConnectionWindow,
    HideConnectionWindow,

    MarkWindowAsServer,
    MarkWindowAsClient,
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
fn listen_connection_window_input(rec: &Receiver<ConnectionOptions>)->ConnectionOptions{
    loop {
        std::thread::sleep(std::time::Duration::from_millis(5));
        if let Ok(msg) = rec.recv(){
            return msg;
        }
    }
}

// mut res: BattleResultWindow,
fn handle_game(
    sender: AppSender<GUIEvents>,
    prep_reciever: Receiver<BattlePreparationEvents>,
    bat_reciever: Receiver<BattleWindowEvents>,
    res_reciever: Receiver<BattleResultsEvents>,
    conn_window: Receiver<ConnectionOptions>
) {

    
    let mut my_move = false;
    
    let conn: Connection;
    let mut bat_num=1;
    sender.send(GUIEvents::ShowConnectionWindow);
    match listen_connection_window_input(&conn_window){
        ConnectionOptions::ConnectAsServer(numb)=>{
            sender.send(GUIEvents::HideConnectionWindow);
            
            bat_num=numb;

            conn=Connection::connect_as_server().unwrap();
            conn.write(Message { data: [0,bat_num]});                    
        
            sender.send(GUIEvents::MarkWindowAsServer);
        }
        ConnectionOptions::ConnectAsClient(addr)=>{                 
            sender.send(GUIEvents::HideConnectionWindow);

            my_move=true;

            conn=Connection::connect_as_client(&format!("{}:8888",addr.to_string())).unwrap();   

            


            let dat= conn.listen().join().unwrap().unwrap().data;
            match dat{
                [0,_]=>{
                    bat_num=dat[1];
                }
                _=>()    
            };

            sender.send(GUIEvents::MarkWindowAsClient);

        }
    }


    for _ in 0..bat_num {
        my_move = !my_move;
        
        
        sender.send(GUIEvents::ShowPreparationsWindow);
        let result = conn.listen_for(Message { data: [0, 0] });
        
        let mut player_field = prepare_field(sender.clone(), &prep_reciever);
        
        sender.send(GUIEvents::HidePreparationsWindow);
        
        conn.write(Message { data: [0, 0] });
        
        match result.join().unwrap() {
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

                match conn.listen().join().unwrap() {
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
                match conn.listen().join().unwrap() {
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

fn main() {
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
    
    let (connection_window_sender, connection_window_reciever): (
        Sender<ConnectionOptions>,
        Receiver<ConnectionOptions>,
    ) = std::sync::mpsc::channel();
    

    
    
    
    let mut battle_window = BattleWindow::new();
    battle_window.set_handler(battle_window_sender);
    battle_window.hide();
    wind.add(&battle_window.group);
    
    let mut battle_prep_window = BattlePrepWindow::new();
    battle_prep_window.set_handler(battle_prep_sender);
    battle_prep_window.hide();    
    wind.add(&battle_prep_window.group);

    let mut battle_results_window = BattleResultWindow::new();
    battle_results_window.set_handler(result_window_sender);
    battle_results_window.hide();
    wind.add(&battle_results_window.group);

    let mut connection_window = ConnectionWindow::new();
    connection_window.set_handler(connection_window_sender);
    connection_window.hide();
    wind.add(&connection_window.flex);


    wind.make_resizable(true);

    
    

    
    


    let handle = std::thread::spawn(move ||{
        handle_game(
        sender,
        battle_prep_reciever,
        battle_window_reciever,
        result_window_reciever,
        connection_window_reciever
        );
    });
    wind.show();
   
    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            app.quit();
        }
    });


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

                GUIEvents::ShowConnectionWindow=>{
                    connection_window.show();
                    // wind.redraw();
                }

                GUIEvents::HideConnectionWindow=>{
                    connection_window.hide();
                    // wind.redraw();

                }

                GUIEvents::MarkWindowAsClient=>{
                    wind.set_label("SEA-BATTLE-CLIENT");
                }

                
                GUIEvents::MarkWindowAsServer=>{
                    wind.set_label("SEA-BATTLE-SERVER");
                }
            }
        }
    }


    app.run().unwrap();
}
