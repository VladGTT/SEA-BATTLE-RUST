mod battle_prep_window;
mod battle_window;
mod battle_results_window;
mod connection;
mod draw_table;
mod play_field;
mod stats;

use fltk::{
    app::{self, Receiver},
    enums,
    prelude::*,
    window,
};

use stats::{BattleStatistics,BattleResults};

use connection::{Connection, Message};
use play_field::{Field, GameField, PlayField, PrepField, StrikeResponce};

use battle_prep_window::{BattlePrepWindow, BattlePreparationEvents};
use battle_window::{BattleWindow, BattleWindowEvents};
use battle_results_window::*;

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET: &str = "localhost:8888";

fn prepare_field(wind: &mut BattlePrepWindow) -> PlayField {
    let (s, r) = app::channel::<BattlePreparationEvents>();
    let mut field = PlayField::new_player_field();

    wind.set_handler(s);
    wind.draw(&field);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(5));
        if let Some(msg) = r.recv() {
            match msg {
                BattlePreparationEvents::Ready => {
                    if field.get_ship_numb() != (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK) {
                        continue;
                    }
                    wind.draw(&mut field);
                    break;
                }
                BattlePreparationEvents::Reset => {
                    field.reset();
                    wind.draw(&mut field);
                }
                BattlePreparationEvents::ShipPlaced(coords) => {
                    field.place_ship(coords);
                    wind.draw(&mut field);
                }
            }
        }
    }
    field
}

fn listen_player_strike(r: &Receiver<BattleWindowEvents>) -> (i32, i32) {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        if let Some(msg) = r.recv() {
            if let BattleWindowEvents::Strike(coords) = msg {
                return coords;
            }
        }
    }
}
fn show_battle_results(wind: &mut BattleResultWindow){
    let (s, r) = app::channel::<BattleResultsEvents>();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        if let Some(msg) = r.recv() {
            if let BattleResultsEvents::ButtonPressed = msg {
                return;
            }
        }
    }
}
// mut res: BattleResultWindow,
async fn handle_game(is_server: bool,mut prep: BattlePrepWindow,mut bat: BattleWindow,conn: Connection) {
    
    let mut my_move = !is_server;
    for _ in 0..3{
        my_move=!my_move;


        // Prepare player field
        bat.hide();
        // res.hide();
        
        let result = conn.listen_for(&Message { data: [0, 0] });
        
        prep.show();
        let mut player_field = prepare_field(&mut prep);
        prep.hide();
    
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
        let (s, r) = app::channel::<BattleWindowEvents>();
    
        bat.show();
        bat.set_handler(s);
        bat.draw(&player_field, &opponent_field);
        bat.group.redraw();
         
        let mut stats = BattleStatistics{
            player_shots_hit:0,
            player_shots_fired:0,
            opponent_shots_hit:0,
            opponent_shots_fired:0,
            player_ships_destroed: (0,0,0,0),
            opponent_ships_destroed: (0,0,0,0),
            results: None
        };
        loop {
            if stats.results.is_some(){
                if let BattleResults::PlayerWon = stats.results.as_ref().unwrap() {
                    println!("You won");
                }else {
                    println!("You lost");
                }
                break;
            }
            if my_move {
                let coords = listen_player_strike(&r);
                bat.opponent_field.deactivate();
                conn.write(Message {data: [coords.0 as u8, coords.1 as u8]});

                stats.player_shots_fired+=1;

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
                            stats.player_shots_hit+=1;
                            
                        }
                        [254, 254] => {
                            opponent_field.mark_as_hit((coords.0 as u8, coords.1 as u8));
                            stats.player_shots_hit+=1;
                            
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
                            stats.player_shots_hit+=1;
    
                            stats.results=Some(BattleResults::PlayerWon);
                        }
                        _ => (),
                    },
                }
                bat.opponent_field.activate();
                
            } else {
                bat.opponent_field.deactivate();
                
                stats.opponent_shots_fired+=1;
                match conn.listen().await {
                    None => {
                        println!("Connection_broken");
                    }
                    Some(data) => match player_field.strike_coords((data.data[0], data.data[1])) {
                        StrikeResponce::Hit => {
                            conn.write(Message { data: [254, 254] });
                            stats.opponent_shots_hit+=1;
                        }
                        StrikeResponce::Kill => {
                            conn.write(Message { data: [255, 255] });
                            stats.opponent_shots_hit+=1;

                        }
                        StrikeResponce::Miss => {
                            conn.write(Message { data: [253, 253] });
                            my_move = true;
                        }
                        StrikeResponce::KilledLast => {
                            conn.write(Message { data: [252, 252] });
                            stats.opponent_shots_hit+=1;                            
                            stats.results=Some(BattleResults::PlayerLost);
                        }
                    },
                }
                bat.opponent_field.activate();
            }
            bat.draw(&player_field, &opponent_field);
        }
        bat.hide();
        // res.draw(&stats);
        // res.show();
        // show_battle_results(&mut res);
    }
    
    
    
}

#[tokio::main]
async fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = window::Window::default().with_size(800, 600);


    let mut battle_prep_window = BattlePrepWindow::new();

    // let battle_results_window = BattleResultWindow::new();
    
    let mut battle_window = BattleWindow::new();

    wind.make_resizable(true);
    wind.end();

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

    let mode_str = &mode[..mode.len() - 2];

    let mut connection: Option<Connection> = None;
    if mode_str == "server" {
        connection.insert(Connection::connect_as_server(SOCKET).unwrap());
        wind.set_label("SEA-BATTLE-SERVER");
    } else {
        connection.insert(Connection::connect_as_client(SOCKET).unwrap());
        wind.set_label("SEA-BATTLE-CLIENT");
    };

    let handle = tokio::spawn(handle_game(
        mode_str == "server",
        battle_prep_window,
        battle_window,
        // battle_results_window,
        connection.unwrap().clone(),
    ));

    wind.show();
    app.run().unwrap();
}
