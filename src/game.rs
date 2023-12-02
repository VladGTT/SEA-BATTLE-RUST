use crate::play_field::{Field, GameField, PlayField, PrepField, StrikeResponce};
use crate::stats::{BattleResults, BattleStatistics};

use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;

use crate::battle_prep_window::BattlePreparationEvents;
use crate::battle_results_window::*;
use crate::battle_window::BattleWindowEvents;
use crate::connection_window::ConnectionOptions;

use crate::connection::{Connection, Message};
use crate::ui::GUIEvents;

use fltk::app::Sender as AppSender;

pub const MAX_4DECK: i32 = 1;
pub const MAX_3DECK: i32 = 2;
pub const MAX_2DECK: i32 = 3;
pub const MAX_1DECK: i32 = 4;

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
fn listen_connection_window_input(rec: &Receiver<ConnectionOptions>) -> ConnectionOptions {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(5));
        if let Ok(msg) = rec.recv() {
            return msg;
        }
    }
}











pub fn handle_game(
    sender: AppSender<GUIEvents>,
    prep_reciever: Receiver<BattlePreparationEvents>,
    bat_reciever: Receiver<BattleWindowEvents>,
    res_reciever: Receiver<BattleResultsEvents>,
    conn_window: Receiver<ConnectionOptions>,
)->JoinHandle<()> {
    std::thread::spawn(move || {
        let mut my_move = false;

        let conn: Connection;
        let mut bat_num = 1;
        sender.send(GUIEvents::ShowConnectionWindow);
        match listen_connection_window_input(&conn_window) {
            ConnectionOptions::ConnectAsServer(numb) => {
                sender.send(GUIEvents::HideConnectionWindow);

                bat_num = numb;

                conn = Connection::connect_as_server().unwrap();
                conn.write(Message { data: [0, bat_num] });

                sender.send(GUIEvents::MarkWindowAsServer);
            }
            ConnectionOptions::ConnectAsClient(addr) => {
                sender.send(GUIEvents::HideConnectionWindow);

                my_move = true;

                conn =
                    Connection::connect_as_client(&format!("{}:8888", addr.to_string())).unwrap();

                let dat = conn.listen().join().unwrap().unwrap().data;
                match dat {
                    [0, _] => {
                        bat_num = dat[1];
                    }
                    _ => (),
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
                player_ships_destroed: (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK),
                opponent_ships_destroed: (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK),
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
                        Some(data) => {
                            match player_field.strike_coords((data.data[0], data.data[1])) {
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
                            }
                        }
                    }
                    sender.send(GUIEvents::EnableBattleWindow);
                }
                sender.send(GUIEvents::RedrawBattleWindow(player_field, opponent_field));
            }
            sender.send(GUIEvents::HideBattleWindow);

            {
                stats.opponent_ships_destroed.0 -= opponent_field.get_ship_numb().0;
                stats.opponent_ships_destroed.1 -= opponent_field.get_ship_numb().1;
                stats.opponent_ships_destroed.2 -= opponent_field.get_ship_numb().2;
                stats.opponent_ships_destroed.3 -= opponent_field.get_ship_numb().3;

                stats.player_ships_destroed.0 -= player_field.get_ship_numb().0;
                stats.player_ships_destroed.1 -= player_field.get_ship_numb().1;
                stats.player_ships_destroed.2 -= player_field.get_ship_numb().2;
                stats.player_ships_destroed.3 -= player_field.get_ship_numb().3;
            }

            sender.send(GUIEvents::RedrawResultsWindow(stats));
            sender.send(GUIEvents::ShowResultsWindow);
            show_battle_results(&res_reciever);
            sender.send(GUIEvents::HideResultsWindow);
        }
    })
}
