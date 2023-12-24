use crate::play_field::{Field, GameField, PlayField, PrepField, StrikeResponce};
use crate::stats::{BattleStatistics, PlayersRating};

use std::sync::mpsc::{Sender,Receiver};
use std::thread::JoinHandle;

use crate::battle_prep_window::BattlePreparationEvents;
use crate::battle_results_window::*;
use crate::battle_window::BattleWindowEvents;
use crate::connection_window::ConnectionOptions;

use crate::connection::{Connection, Message};
use crate::ui::GUIEvents;

use fltk::app::Sender as AppSender;
use std::{net::Ipv4Addr, str::FromStr};


pub const MAX_4DECK: i32 = 1;
pub const MAX_3DECK: i32 = 2;
pub const MAX_2DECK: i32 = 3;
pub const MAX_1DECK: i32 = 4;


#[derive(Clone, Copy)]
pub enum GameEvents{
    PlayerSurrendered,
    OpponentSurrendered,

    ConnectAs(ConnectionOptions),

    Strike((i32,i32)),

    ShipPlaced((i32,i32,i32,i32)),
    Ready,
    Reset,

    ToNextBattle,


    Hit,
    Killed,
    KilledLast,
    Missed,

    NumbOfBattes(i32),

    OpponentStrike((i32,i32)),
    OpponnentReady
} 








pub fn handle_game(
    sender: AppSender<GUIEvents>,
    recv: Receiver<GameEvents>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut my_move = false;


        let mut bat_num = 1;
        sender.send(GUIEvents::ShowConnectionWindow);
        let mut conn: Option<Connection> = None;
        if let Ok(msg) = recv.recv(){
            if let GameEvents::ConnectAs(options) = msg{
                match options{
                    ConnectionOptions::ConnectAsServer(numb)=>{
                        sender.send(GUIEvents::HideConnectionWindow);
    
                        bat_num = numb;    
                        let mut c = Connection::connect_as_server(sender.clone()).unwrap();
                        c.write(Message { data: [50, bat_num] });

                        conn=Some(c);
                        sender.send(GUIEvents::MarkWindowAsServer);
                    }
                    ConnectionOptions::ConnectAsClient(addr)=>{
                        sender.send(GUIEvents::HideConnectionWindow);
                    
                        my_move = true;
                        let mut c = Connection::connect_as_client(&format!("{}:8888", addr.to_string()),sender.clone()).unwrap();
                        if let Ok(msg) = recv.recv(){
                            if let GameEvents::NumbOfBattes(numb) = msg {
                                bat_num = numb as u8;
                            }
                        }
                        conn=Some(c);
                        sender.send(GUIEvents::MarkWindowAsClient);
                    }
                }

            }
        }

        



        let mut rating = PlayersRating {
            n_wins_player: 0,
            n_wins_opponent: 0,
        };

        for _ in 0..bat_num {
            my_move = !my_move;

            sender.send(GUIEvents::ShowPreparationsWindow);


            let mut player_field = PlayField::new_player_field();

            sender.send(GUIEvents::RedrawPreparationsWindow(player_field));
        
            let mut is_opponent_ready=false;
            let mut is_player_ready=false;

            while !is_player_ready || !is_opponent_ready {
                if let Ok(msg) = recv.recv_timeout(std::time::Duration::from_millis(5)) {
                    match msg {
                        GameEvents::Ready => {
                            if player_field.get_ship_numb() == (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK){
                                is_player_ready=true;
                                conn.as_ref().unwrap().write(Message{data: [200,200]});
                                sender.send(GUIEvents::RedrawPreparationsWindow(player_field));
                                sender.send(GUIEvents::HidePreparationsWindow);

                            }
                        }
                        GameEvents::Reset => {
                            player_field.reset();
                        }
                        GameEvents::ShipPlaced(coords) => {
                            player_field.place_ship(coords);
                            sender.send(GUIEvents::RedrawPreparationsWindow(player_field));
                        }
                        GameEvents::OpponnentReady => {
                            is_opponent_ready=true;
                        }
                        _=>()
                    }
                }
            }











            // Play battle
            let mut opponent_field = PlayField::new_opponent_field();

            sender.send(GUIEvents::RedrawBattleWindow(player_field, opponent_field));

            sender.send(GUIEvents::ShowBattleWindow);

            let mut player_stats = BattleStatistics {
                player_shots_hit: 0,
                player_shots_fired: 0,
                player_ships_destroed: (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK),
                player_won: None,
            };

            let mut opponent_stats = BattleStatistics {
                player_shots_hit: 0,
                player_shots_fired: 0,
                player_ships_destroed: (MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK),
                player_won: None,
            };




            'game: loop {
                

                if player_stats.player_won.is_some() || opponent_stats.player_won.is_some() {
                    break;
                }


                if my_move {
                    
                    let coords: (i32,i32);
                    loop {
                        if let Ok(msg) = recv.recv_timeout(std::time::Duration::from_millis(10)) {
                            if let GameEvents::Strike(crds) = msg {
                                coords=crds;
                                break;
                            }
                            if let GameEvents::PlayerSurrendered = msg{
                                rating.n_wins_opponent += 1;
                                opponent_stats.player_won = Some(true);
                                player_stats.player_won = Some(false);
                                conn.as_ref().unwrap().write(Message { data: [127,127] });
                                println!("Player surrendered");
                                break 'game;
                            }
                            if let GameEvents::OpponentSurrendered = msg{
                                rating.n_wins_player += 1;
                                opponent_stats.player_won = Some(false);
                                player_stats.player_won = Some(true);
                                println!("Opponent surrendered");
                                break 'game;
                            } 
                        }
                    }    





                    sender.send(GUIEvents::DisableBattleWindow);

                    conn.as_ref().unwrap().write(Message {data: [coords.0 as u8, coords.1 as u8]});

                    player_stats.player_shots_fired += 1;

                    
                    loop {
                        if let Ok(msg) = recv.recv_timeout(std::time::Duration::from_millis(10)) {
                            if let GameEvents::PlayerSurrendered = msg{
                                rating.n_wins_opponent += 1;
                                opponent_stats.player_won = Some(true);
                                player_stats.player_won = Some(false);
                                conn.as_ref().unwrap().write(Message { data: [127,127] });
                                println!("Player surrendered");
                                break 'game;
                            }
                            if let GameEvents::OpponentSurrendered = msg{
                                rating.n_wins_player += 1;
                                opponent_stats.player_won = Some(false);
                                player_stats.player_won = Some(true);
                                println!("Opponent surrendered");
                                break 'game;
                            }
                            if let GameEvents::Hit = msg{
                                opponent_field.mark_as_hit((coords.0 as u8, coords.1 as u8));
                                player_stats.player_shots_hit += 1;
                                break;
                            } 
                            if let GameEvents::Missed  = msg{
                                opponent_field.mark_as_miss((coords.0 as u8, coords.1 as u8));
                                my_move = false;
                                break;

                            } 
                            if let GameEvents::Killed  = msg{
                                opponent_field.mark_as_kill(
                                    opponent_field
                                        .check_if_killed((coords.0 as u8, coords.1 as u8))
                                        .unwrap(),
                                );
                                player_stats.player_shots_hit += 1;    
                                break;

                            } 
                            if let GameEvents::KilledLast  = msg{
                                opponent_field.mark_as_kill(
                                    opponent_field
                                        .check_if_killed((coords.0 as u8, coords.1 as u8))
                                        .unwrap(),
                                );
                                player_stats.player_shots_hit += 1;
    
                                player_stats.player_won = Some(true);
                                opponent_stats.player_won = Some(false);
    
                                rating.n_wins_player += 1;    
                                break;

                            } 
                        }
                    }


                    sender.send(GUIEvents::EnableBattleWindow);
                } else {
                    sender.send(GUIEvents::DisableBattleWindow);

                    opponent_stats.player_shots_fired += 1;

                    let mut data: (i32,i32);
                    loop {
                        if let Ok(msg) = recv.recv_timeout(std::time::Duration::from_millis(10)) {
                            if let GameEvents::PlayerSurrendered = msg{
                                rating.n_wins_opponent += 1;
                                opponent_stats.player_won = Some(true);
                                player_stats.player_won = Some(false);
                                conn.as_ref().unwrap().write(Message { data: [127,127] });
                                println!("Player surrendered");
                                break 'game;
                            }
                            if let GameEvents::OpponentSurrendered = msg{
                                rating.n_wins_player += 1;
                                opponent_stats.player_won = Some(false);
                                player_stats.player_won = Some(true);
                                println!("Opponent surrendered");
                                break 'game;
                            } 
                            if let GameEvents::OpponentStrike(dat) = msg{
                                data=dat;
                                break;
                            }
                        }


                    }


                    
                    match player_field.strike_coords((data.0 as u8, data.1 as u8)) {
                        StrikeResponce::Hit => {
                            conn.as_ref().unwrap().write(Message { data: [254, 254] });
                            opponent_stats.player_shots_hit += 1;
                        }
                        StrikeResponce::Kill => {
                            conn.as_ref().unwrap().write(Message { data: [255, 255] });
                            opponent_stats.player_shots_hit += 1;
                        }
                        StrikeResponce::Miss => {
                            conn.as_ref().unwrap().write(Message { data: [253, 253] });
                            my_move = true;
                        }
                        StrikeResponce::KilledLast => {
                            conn.as_ref().unwrap().write(Message { data: [252, 252] });
                            opponent_stats.player_shots_hit += 1;
                            opponent_stats.player_won = Some(true);
                            player_stats.player_won = Some(false);

                            rating.n_wins_opponent += 1;
                        }
                    }
                    sender.send(GUIEvents::EnableBattleWindow);
                }
                sender.send(GUIEvents::RedrawBattleWindow(player_field, opponent_field));
            }





            sender.send(GUIEvents::HideBattleWindow);

            player_stats.calc_ships_destroed(player_field.get_ship_numb());
            opponent_stats.calc_ships_destroed(opponent_field.get_ship_numb());

            sender.send(GUIEvents::RedrawResultsWindow(
                player_stats,
                opponent_stats,
                rating,
            ));
            sender.send(GUIEvents::ShowResultsWindow);







            
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
                if let Ok(msg) = recv.recv() {
                    if let GameEvents::ToNextBattle = msg {
                        break;
                    }
                }
            }
                     
            
            
            
            
            
            
            
            
            
            
            sender.send(GUIEvents::HideResultsWindow);
        }




        sender.send(GUIEvents::ShowGameResults(
            rating.n_wins_player,
            rating.n_wins_opponent,
        ));
    })
}
