mod play_field;
mod my_window;
mod connection;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use fltk::{window,enums,app,prelude::*};

use play_field::PlayField;
use my_window::{MyWindow,PrepareWindow,MatchWindow,Visible};
use connection::Connection;

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const SOCKET_INPUT: &str = "localhost:8888"; 
const SOCKET_OUTPUT: &str = "localhost:8889"; 

#[derive(Clone,Copy)]
enum CustomEvents{
    Ready,
    ShipPlaced,
    ResetField,
    WindowClosed,
    PlayerStrikes,
    ConnectAsServer,
    ConnectAsClient,
}    

static PLAYER_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(PlayField::default()));
static OPPONNENT_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(PlayField::default()));

static INPUT: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));
static OUTPUT: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));

static CURRENT_MATCH: Lazy<Mutex<Match>> = Lazy::new(|| Mutex::new(Match::default()));


struct Match{
    player_ready: bool,
    opponent_ready: bool,
    is_server: bool,

    last_coords: (u8,u8)
}



impl Default for Match {
    fn default() -> Self {
        Match {player_ready:false,opponent_ready:false,is_server:false,last_coords: (255,255)}
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
        CURRENT_MATCH.lock().unwrap().is_server=true;
    } else {
        s.send(CustomEvents::ConnectAsClient);        
    }

    

    let pt_callback = |(r,c):(i32,i32)|->u8{
        let field = &mut PLAYER_FIELD.lock().unwrap();
        field.field[r as usize][c as usize]
    };
    let ot_callback = |(r,c):(i32,i32)|->u8{
        let field = &mut OPPONNENT_FIELD.lock().unwrap();
        field.field[r as usize][c as usize]
    };



    let handle_strike = |buf: &[u8;3]|{
        match buf {
            [_,255,255] => {
                println!("Hit");
            },
            [_,254,254] => {
                println!("Miss");
            },
            [253,253,253] => {
                let mut current_match = CURRENT_MATCH.lock().unwrap();
                current_match.opponent_ready=true;
                println!("Opponent is ready");
            },
            _=>{
                println!("Message delivered successfully");
                let mut player_field=PLAYER_FIELD.lock().unwrap();
                let mut output = OUTPUT.lock().unwrap();
                // let mut current_match = CURRENT_MATCH.lock().unwrap();
                
                match player_field.strike((buf[1],buf[2])){
                    Ok(confirm)=>{
                        output.write(&[1,255,255]);
                    },
                    Err(_)=>{
                        output.write(&[2,254,254]);
                    }
                }
                
            }
        }
    };

    let handle_player_strike = |coords: (u8,u8)|{
        let mut current_match = CURRENT_MATCH.lock().unwrap();
        current_match.last_coords=coords;
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
                    println!("Ready");
                    
                    let mut current_match = CURRENT_MATCH.lock().unwrap();

                    current_match.player_ready=true;
                    
                    let mut output=OUTPUT.lock().unwrap();
                    output.write(&[253,253,253]);

                    prep_window.hide();
                    match_window.show();
                    
                },
                CustomEvents::WindowClosed=>{
                    println!("Closed");
                    app.quit();
                },
                CustomEvents::PlayerStrikes=>{
                    println!("Player strikes");

                    let mut current_match = CURRENT_MATCH.lock().unwrap();
                    
                    if !(current_match.player_ready && current_match.opponent_ready) {continue;}


                    let mut output=OUTPUT.lock().unwrap();


                    output.write(&[30,current_match.last_coords.0,current_match.last_coords.1]);
                },
                CustomEvents::ConnectAsServer=>{
                    
                    std::thread::spawn(move ||{
                        let mut input = INPUT.lock().unwrap();
                        match input.connect_as_server(SOCKET_INPUT){
                            Ok(_)=>println!("Input connected"),
                            Err(_)=>println!("Input connection error")
                        }

                        input.listen(handle_strike);
                    });
                    std::thread::spawn(move ||{
                        let mut output = OUTPUT.lock().unwrap();
                        match output.connect_as_server(SOCKET_OUTPUT){
                            Ok(_)=>println!("Output connected"),
                            Err(_)=>println!("Output connection error")
                        }
                    });
                },
                CustomEvents::ConnectAsClient=>{
                    std::thread::spawn(move ||{
                        let mut input = INPUT.lock().unwrap();
                        match input.connect_as_client(SOCKET_OUTPUT){
                            Ok(_)=>println!("Input connected"),
                            Err(_)=>println!("Input connection error")
                        }

                        input.listen(handle_strike);
                    });
                    std::thread::spawn(move ||{
                        let mut output = OUTPUT.lock().unwrap();
                        match output.connect_as_client(SOCKET_INPUT){
                            Ok(_)=>println!("Output connected"),
                            Err(_)=>println!("Output connection error")
                        }
                    });
                },
            }
        }
    };
    
    app.run().unwrap();
}
