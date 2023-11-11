mod play_field;
mod my_window;
mod connection;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use fltk::{window,enums,app,prelude::*};
// use std::{thread,time};

use play_field::PlayField;
use my_window::{MyWindow,PrepareWindow,MatchWindow,Visible};
use connection::Connection;

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET: &str = "localhost:8888"; 
// const SOCKET_OUTPUT: &str = "localhost:8889"; 

#[derive(Clone,Copy)]
pub enum CustomEvents{
    Ready,
    ShipPlaced,
    ResetField,
    WindowClosed,
    PlayerStrikes,
    OpponentStrikes,
    ConnectAsServer,
    ConnectAsClient,
}    

static PLAYER_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(PlayField::default()));
static OPPONNENT_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(PlayField::default()));

static CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));

static CURRENT_MATCH: Lazy<Mutex<Match>> = Lazy::new(|| Mutex::new(Match::default()));


struct Match{
    player_ready: bool,
    opponent_ready: bool,
    my_move: bool,

    last_coords: (u8,u8)
}



impl Default for Match {
    fn default() -> Self {
        Match {player_ready:false,opponent_ready:false,my_move:false,last_coords: (255,255)}
    }
}

fn main() {

    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let (s, r) = app::channel::<CustomEvents>();

    let mut mode = String::new();

    let stdin = std::io::stdin();

    println!("Write mode");
    
    stdin.read_line(&mut mode).unwrap();

    let mode_str=&mode[..mode.len()-2];


    if mode_str == "server" {
        s.send(CustomEvents::ConnectAsServer);
    } else {
        s.send(CustomEvents::ConnectAsClient);        
    }

    

    let pt_callback = |(r,c):(i32,i32)|->u8{
        PLAYER_FIELD.lock().unwrap().field[r as usize][c as usize]
    };
    let ot_callback = |(r,c):(i32,i32)|->u8 {
        OPPONNENT_FIELD.lock().unwrap().field[r as usize][c as usize]
    };



    let handle_strike = |buf: &[u8;3]|{
        match buf {
            [_,255,255] => {
                println!("Hit");
                
                
                let mut current_match = CURRENT_MATCH.lock().unwrap();
                
                current_match.my_move=true;

                OPPONNENT_FIELD.lock().unwrap().mark_as_hit(current_match.last_coords);
            },
            [_,254,254] => {

                println!("Miss");
                let mut current_match = CURRENT_MATCH.lock().unwrap();
                OPPONNENT_FIELD.lock().unwrap().mark_as_miss(current_match.last_coords);
            },
            [253,253,253] => {
                CURRENT_MATCH.lock().unwrap().opponent_ready=true;
                println!("Opponent is ready");

            },
            _=>{
                match PLAYER_FIELD.lock().unwrap().strike((buf[1],buf[2])){
                    Ok(_)=>{
                        CONNECTION.lock().unwrap().write(&[1,255,255]);
                    },
                    Err(_)=>{
                        CONNECTION.lock().unwrap().write(&[2,254,254]);
                        CURRENT_MATCH.lock().unwrap().my_move=true;
                    }
                }
                
            }
        }
    };

    let handle_player_strike = |coords: (u8,u8)|{
        CURRENT_MATCH.lock().unwrap().last_coords=coords;
    };

    let mut wind = window::Window::default().with_size(800, 600);
    wind.set_label("SEA BATTLE");
    let mut prep_window = MyWindow::new_prep_window(s,pt_callback);

    let mut match_window = MyWindow::new_match_window(s,pt_callback,ot_callback,handle_player_strike);
    match_window.hide();

    wind.make_resizable(true);
    wind.end();
    wind.show();

    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            s.send(CustomEvents::WindowClosed)
        }
    });
    

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                CustomEvents::ShipPlaced =>{
                    prep_window.place_ship(|(rt, cl, rb, cr)|->(i32,i32,i32,i32){                        
                        let mut field = &mut PLAYER_FIELD.lock().unwrap();
                        let _ = PlayField::place_ship(&mut field,(rt, cl, rb, cr));
                        return field.get_ship_numb();
                    });
                },
                CustomEvents::ResetField=>{
                    let mut player_field=PLAYER_FIELD.lock().unwrap();
                    player_field.reset();   
                    prep_window.reset();
                },
                CustomEvents::Ready=>{
                    

                    if PLAYER_FIELD.lock().unwrap().get_ship_numb() != (MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK){
                        println!("Ships are not placed");
                        continue;
                    }

                    CURRENT_MATCH.lock().unwrap().player_ready=true;




                    CONNECTION.lock().unwrap().write(&[253,253,253]);


                    prep_window.hide();
                    match_window.show();
                    
                    println!("Ready");
                    
                },
                CustomEvents::OpponentStrikes=>{
                    match_window.group.redraw();
                }
                CustomEvents::WindowClosed=>{
                    println!("Closed");
                    app.quit();
                },
                CustomEvents::PlayerStrikes=>{
                    
                    
                    let mut current_match = CURRENT_MATCH.lock().unwrap();
                    
                    if !(current_match.player_ready && current_match.opponent_ready && current_match.my_move) {continue;}

                    current_match.my_move=false;

                    
                    CONNECTION.lock().unwrap().write(&[30,current_match.last_coords.0,current_match.last_coords.1]);
                    


                    println!("Player strikes");
                },
                CustomEvents::ConnectAsServer=>{
                    match CONNECTION.lock().unwrap().connect_as_server(SOCKET,handle_strike,s){
                        Ok(_)=>println!("Connected"),
                        Err(_)=>println!("Connection error")
                    }
                    CURRENT_MATCH.lock().unwrap().my_move=true;
                },
                CustomEvents::ConnectAsClient=>{
                    match CONNECTION.lock().unwrap().connect_as_client(SOCKET,handle_strike,s){
                        Ok(_)=>println!("Connected"),
                        Err(_)=>println!("Connection error")
                    }
                },
            }
        }
    };
    
    app.run().unwrap();
}
