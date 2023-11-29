mod battle;
mod battle_prep_window;
mod battle_window;
mod connection;
mod draw_table;
mod play_field;

use fltk::{
    app::{self},
    enums,
    prelude::*,
    window::{self, Window},
};

use tokio::*;


use connection::{Connection, Message};
use play_field::{Field, GameField, PlayField, PrepField, StrikeResponce};

use battle_prep_window::{BattlePrepWindow, BattlePreparationEvents};
use battle_window::{BattleWindow, BattleWindowEvents};

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET: &str = "localhost:8888";

fn prepare_field(mut wind: BattlePrepWindow) -> PlayField {
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
                    wind.group.hide();
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

async fn handle_game(
    is_server: bool,
    prep: BattlePrepWindow,
    mut bat: BattleWindow,
    mut conn: Connection,
) {
    let mut my_move = is_server;

    // Prepare player field

    let res = conn.listen_for(&Message { data: [0, 0] });

    let mut player_field = prepare_field(prep);

    conn.write(Message { data: [0, 0] });

    match res.await {
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

    bat.set_handler(s);
    bat.draw(&player_field, &opponent_field);
    bat.group.show();
    
    loop {
        if my_move{
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
                if let Some(msg) = r.recv() {
                    if let BattleWindowEvents::Strike(coords) = msg {
                        bat.opponent_field.deactivate();

                        conn.write(Message {
                            data: [coords.0 as u8, coords.1 as u8],
                        });
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
                                }
                                [254, 254] => {
                                    opponent_field.mark_as_hit((coords.0 as u8, coords.1 as u8));
                                }
                                [253, 253] => {
                                    opponent_field.mark_as_miss((coords.0 as u8, coords.1 as u8));
                                    my_move = false;
                                }
                                _ => (),
                            },
                        }
                        bat.opponent_field.activate();
                    }
                    break;
                }
            }
        }
        else{
            bat.opponent_field.deactivate();

            match conn.listen().await {
                None => {
                    println!("Connection_broken");
                }
                Some(data) => match player_field.strike_coords((data.data[0], data.data[1])) {
                    StrikeResponce::Hit => {
                        player_field.mark_as_hit((data.data[0], data.data[1]));
                        conn.write(Message { data: [254, 254] });
                    }
                    StrikeResponce::Kill(coords) => {
                        player_field.mark_as_kill(coords);
                        conn.write(Message { data: [255, 255] });
                    }
                    StrikeResponce::Miss => {
                        player_field.mark_as_miss((data.data[0], data.data[1]));
                        conn.write(Message { data: [253, 253] });
                        my_move = true;
                    }
                },
            }
            bat.opponent_field.activate();
        }
        bat.draw(&player_field, &opponent_field);
    }
    
    
    
    
    
    
    
    
    
    
    
    
}

#[tokio::main]
async fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = window::Window::default().with_size(800, 600);
    // let (s, r) = app::channel::<bool>();

    let mut battle_prep_window = BattlePrepWindow::new();
    battle_prep_window.group.show();

    let mut battle_window = BattleWindow::new();
    battle_window.group.hide();

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
        connection.unwrap().clone(),
    ));

    wind.show();
    app.run().unwrap();
}
