mod play_field;
mod my_window;
mod connection;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use fltk::{window,enums,app,prelude::*};

use play_field::PlayField;
use my_window::{MyWindow,PrepareWindow,MatchWindow,Visible};
use connection::Connection;

#[derive(Clone,Copy)]
pub enum GameEventType{
    PlayerHits,
    PlayerMisses,
    OpponentStrikes,
    OpponentReady,
    PlayerReady,
    ShipPlaced,
    ResetField,
    WindowClosed,
    PlayerStrikes,
    ConnectAsServer,
    ConnectAsClient,
}

#[derive(Clone,Copy)]
pub struct GameEvent{
    pub event_type: GameEventType,
    pub data: Option<[u8;2]>
}

impl GameEventType{
   pub fn type_to_data(arg:GameEventType)->Result<[u8;2],()>{
        match arg{
            GameEventType::PlayerHits=>Ok([255,255]),
            GameEventType::PlayerMisses=>Ok([254,254]),
            GameEventType::OpponentReady=>Ok([253,253]),
            _ => Err(())
        }
    }
}

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET: &str = "localhost:8888"; 


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

    let (s, r) = app::channel::<GameEvent>();




    {
        let mut mode = String::new();
    
        let stdin = std::io::stdin();
    
        println!("Write mode");
        
        stdin.read_line(&mut mode).unwrap();
    
        let mode_str=&mode[..mode.len()-2];
    
    
        if mode_str == "server" {
            s.send(GameEvent{event_type: GameEventType::ConnectAsServer,data: None});
        } else {
            s.send(GameEvent{event_type: GameEventType::ConnectAsClient,data: None});        
        }
    }


    

    let pt_callback = |(r,c):(i32,i32)|->u8{
        PLAYER_FIELD.lock().unwrap().field[r as usize][c as usize]
    };
    let ot_callback = |(r,c):(i32,i32)|->u8 {
        OPPONNENT_FIELD.lock().unwrap().field[r as usize][c as usize]
    };

    
    
    
    
    let mut wind = window::Window::default().with_size(800, 600);
    wind.set_label("SEA BATTLE");


    let mut prep_window = MyWindow::new_prep_window(s,pt_callback);

    let mut match_window = MyWindow::new_match_window(s,pt_callback,ot_callback);
    match_window.hide();

    wind.make_resizable(true);
    wind.end();
    wind.show();

    wind.set_callback(move |_| {
        if app::event() == enums::Event::Close {
            s.send(GameEvent { event_type: GameEventType::WindowClosed, data: None })
        }
    });
    
    while app.wait(){
        if let Some(msg) = r.recv() {
            match msg.event_type {
                GameEventType::ShipPlaced =>{
                    prep_window.place_ship(|(rt, cl, rb, cr)|->(i32,i32,i32,i32){                        
                        let mut field = &mut PLAYER_FIELD.lock().unwrap();
                        let _ = PlayField::place_ship(&mut field,(rt, cl, rb, cr));
                        field.get_ship_numb()
                    });
                },
                GameEventType::ResetField=>{
                    let mut player_field=PLAYER_FIELD.lock().unwrap();
                    player_field.reset();   
                    prep_window.reset();
                },
                GameEventType::PlayerReady=>{
                    
                    if PLAYER_FIELD.lock().unwrap().get_ship_numb() != (MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK){
                        println!("Ships are not placed");
                    }
                    else{
                        CURRENT_MATCH.lock().unwrap().player_ready=true;
                        CONNECTION.lock().unwrap().write(&GameEventType::type_to_data(GameEventType::OpponentReady).unwrap());
                        
                        
                        prep_window.hide();
                        match_window.show();
                                            
                        println!("Ready");
                    }
                    
                },                
                GameEventType::WindowClosed=>{
                    println!("Closed");
                    app.quit();
                },
                GameEventType::PlayerStrikes=>{
                    
                    

                    let mut current_match = CURRENT_MATCH.lock().unwrap();
                    
                    let coords = msg.data.unwrap();
                    current_match.last_coords=(coords[0],coords[1]);

                    if current_match.player_ready && current_match.opponent_ready && current_match.my_move {

                        current_match.my_move=false;
                        CONNECTION.lock().unwrap().write(&[current_match.last_coords.0,current_match.last_coords.1]);
                        println!("Player strikes");
                    
                    }
                },
                GameEventType::ConnectAsServer=>{
                    match CONNECTION.lock().unwrap().connect_as_server(SOCKET,s){
                        Ok(_)=>println!("Connected"),
                        Err(_)=>println!("Connection error")
                    }
                    CURRENT_MATCH.lock().unwrap().my_move=true;
                },
                GameEventType::ConnectAsClient=>{
                    match CONNECTION.lock().unwrap().connect_as_client(SOCKET,s){
                        Ok(_)=>println!("Connected"),
                        Err(_)=>println!("Connection error")
                    }
                },
                GameEventType::OpponentReady=>{
                    CURRENT_MATCH.lock().unwrap().opponent_ready=true;
                    println!("Opponent is ready");
                },
                GameEventType::PlayerHits=>{
                    println!("Hit");
                    
                    
                    let mut current_match = CURRENT_MATCH.lock().unwrap();
                        
                    current_match.my_move=true;
        
                    OPPONNENT_FIELD.lock().unwrap().mark_as_hit(current_match.last_coords);
                    match_window.group.redraw();

                },
                GameEventType::PlayerMisses=>{
                    println!("Miss");
                    let mut current_match = CURRENT_MATCH.lock().unwrap();
                    OPPONNENT_FIELD.lock().unwrap().mark_as_miss(current_match.last_coords);
                    
                    match_window.group.redraw();

                },
                GameEventType::OpponentStrikes=>{
                    let buf = msg.data.unwrap();
                    match PLAYER_FIELD.lock().unwrap().strike((buf[0],buf[1])){
                        Ok(_)=>{
                            CONNECTION.lock().unwrap().write(&GameEventType::type_to_data(GameEventType::PlayerHits).unwrap());
                        },
                        Err(_)=>{
                            CONNECTION.lock().unwrap().write(&GameEventType::type_to_data(GameEventType::PlayerMisses).unwrap());
                            CURRENT_MATCH.lock().unwrap().my_move=true;
                        }
                    }
                    match_window.group.redraw();
    
                }
            }
        }
        
    };
    app.run().unwrap();
    
}
