mod battle_prep_window;
mod battle_window;
mod connection;
mod draw_table;
mod battle;
mod play_field;

use fltk::{
    app::{self}, enums,
    prelude::*,
    window::{self, Window},
};

use tokio::*;

use once_cell::sync::Lazy;
use std::sync::Mutex;

use connection::{Connection,Message};
use play_field::{Field, GameField, PlayField, PrepField, StrikeResponce};


use battle::{Battle};
use battle_prep_window::{BattlePrepWindow, BattlePreparationEvents};
use battle_window::{BattleWindow,BattleWindowEvents};


const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET: &str = "localhost:8888";

// static PLAYER_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(Field::new_player_field()));

// static OPPONNENT_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(Field::new_opponent_field()));

static CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));

static BATTLE: Lazy<Mutex<Option<Battle>>>=Lazy::new(|| Mutex::new(None));





fn prepare_field(wind: &mut BattlePrepWindow, app: &app::App)->PlayField {
    let (s, r) = app::channel::<BattlePreparationEvents>();
    let mut field = PlayField::new_player_field();
    wind.set_handler(s);
    wind.draw(&field);

    
    while app.wait() {
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

#[tokio::main]
async fn main() {
    
    

    let mut my_move = false;

    {
        let mut mode = String::new();

        let stdin = std::io::stdin();

        println!("Write mode");

        stdin.read_line(&mut mode).unwrap();

        let mode_str=&mode[..mode.len()-2];

        if mode_str == "server" {
            match CONNECTION.lock().unwrap().connect_as_server(SOCKET){
                Ok(_)=>println!("Connected"),
                Err(_)=>println!("Connection error")
            }
            my_move=true;

        } else {
            match CONNECTION.lock().unwrap().connect_as_client(SOCKET){
                Ok(_)=>println!("Connected"),
                Err(_)=>println!("Connection error")
            }
        }
    }


    let app = app::App::default().with_scheme(app::Scheme::Gtk);


    let mut wind = window::Window::default().with_size(800, 600);

    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            app.quit();
        }
    });

    let mut prep_window = BattlePrepWindow::new();
    let mut battle_window = BattleWindow::new();
    battle_window.group.hide();

    wind.make_resizable(true);
    wind.end();
    wind.show();

    

    {
        let mut connection=CONNECTION.lock().unwrap();
       
        
        let res = connection.listen_for(&Message { data: [0,0] });
        
        let mut player_field = prepare_field( &mut prep_window, &app);

        connection.write(Message { data: [0,0] });
        
        match res.await{
            Ok(_)=>{
                println!("Opponent_is_ready")
                
            }
            Err(_)=>{
                
                println!("Opponent_is_not_ready");
                return;
            }
        }




        let mut opponent_field = PlayField::new_opponent_field(); 
        let (s, r) = app::channel::<BattleWindowEvents>();


        battle_window.set_handler(s);
        battle_window.draw(&player_field, &opponent_field);
        battle_window.group.show();
        while app.wait() {
            if my_move{
                if let Some(msg) = r.recv() {
                    match msg {
                        BattleWindowEvents::Strike(coords) => {
                            connection.write(Message { data: [coords.0 as u8, coords.1 as u8]});
                            match connection.listen().await{
                                None=>{
                                    println!("Connection_broken");
                                }
                                Some(data)=>{
                                    match data.data {
                                        [255,255]=>{
                                            opponent_field.mark_as_kill(opponent_field.check_if_killed((coords.0 as u8,coords.1 as u8)).unwrap());
                                        }
                                        [254,254]=>{
                                            opponent_field.mark_as_hit((coords.0 as u8,coords.1 as u8));
                                        }
                                        [_,_]=>{
                                            opponent_field.mark_as_miss((coords.0 as u8,coords.1 as u8));
                                            my_move=false;
                                        }
                                    }
                                }
                            }                                                                                   
                        }
                    }
                    battle_window.draw(&player_field, &opponent_field);

                }
            } else {
                match connection.listen().await{
                    None=>{
                        println!("Connection_broken");
                    }
                    Some(data)=>{
                        match player_field.strike_coords((data.data[0],data.data[1])){
                            StrikeResponce::Hit=>{
                                player_field.mark_as_hit((data.data[0],data.data[1]));
                                connection.write(Message { data: [254,254] });
                            },
                            StrikeResponce::Kill(coords)=>{
                                player_field.mark_as_kill(coords);
                                connection.write(Message { data: [255,255] });
                            },
                            StrikeResponce::Miss=>{
                                player_field.mark_as_miss((data.data[0],data.data[1]));
                                connection.write(Message { data: [252,252] });
                                my_move=true;
                            }
                        }
                        battle_window.draw(&player_field, &opponent_field);

                    }
                }
            }
            
        }
    }
    app.run().unwrap();  
}
