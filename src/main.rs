mod play_field;
mod my_window;

use once_cell::sync::Lazy;
use std::sync::{Mutex,MutexGuard};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::{thread,time};

use play_field::PlayField;
use my_window::{MyWindow,PrepareWindow,MatchWindow,Visible};

use fltk::{window,enums,app,prelude::*};

const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;





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



const SOCKET_INPUT: &str = "localhost:8888"; 
const SOCKET_OUTPUT: &str = "localhost:8889"; 



static INPUT: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));
static OUTPUT: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));



// static MATCH: Lazy<Mutex<Match>> = Lazy::new(|| Mutex::new(Match::default()));

// struct Match{
//     is_started:bool,
//     move_numb:Option<i32>,
//     last_strike_coords:Option<(u8,u8)>
// }

// impl Match{
//     fn default()->Self{
//         Match { is_started: false, move_numb: None, last_strike_coords: None }
//     }
// }


struct Connection{
    stream:Option<TcpStream>
}
impl Default for Connection{
    fn default() -> Self {
        Connection { stream: None }
    }
}

impl Connection{
    fn connect(&mut self,str: TcpStream){
        self.stream=Some(str);
    }
    fn read(&mut self,buf: &mut [u8])->Result<(),()>{
        match self.stream{
            Some(ref mut str)=>{
                match str.read(buf){
                    Ok(_)=>Ok(()),
                    Err(_)=>Err(())
                }
            },
            None=>Err(())
        }
    }
    fn write(&mut self,data: &[u8])->Result<(),()>{
        match self.stream{
            Some(ref mut str)=>{
                match str.write(data){
                    Ok(_)=>Ok(()),
                    Err(_)=>Err(())
                }
            },
            None=>Err(())
        }
    }
}


fn connect_as_server(){
    thread::spawn(move ||{
        let mut input = INPUT.lock().unwrap();
        match TcpListener::bind(SOCKET_INPUT).unwrap().accept(){
            Ok((stream,_)) =>{
                input.connect(stream);
                println!("Input connection established");
            },
            Err(_) => println!("Connection input error")
        }

        listen_input(&mut input,handle_strike);
    });

    thread::spawn(||{
        let mut output = OUTPUT.lock().unwrap();
        match TcpListener::bind(SOCKET_OUTPUT).unwrap().accept(){
            Ok((stream,_)) =>{
                output.connect(stream);
                println!("Output connection established");
            },
            Err(_) => println!("Connection output error")
        }                                               
    });
}

fn connect_as_client(){
    thread::spawn(move ||{
        let mut input = INPUT.lock().unwrap();
        match TcpStream::connect(SOCKET_OUTPUT){
            Ok(stream) =>{
                input.connect(stream);
                println!("Input connection established");
            },
            Err(_) => println!("Connection input error")
        }

        listen_input(&mut input,handle_strike);
    });

    
    thread::spawn(||{
        let mut output = OUTPUT.lock().unwrap();
        match TcpStream::connect(SOCKET_INPUT){
            Ok(stream,) => {
                output.connect(stream);
                println!("Output connection established");
            },
            Err(_) => println!("Connection output error")
        }   
    });
}

fn listen_input(input: &mut MutexGuard<'_, Connection>, func: fn(&[u8;3])){
    let mut buf = [0 as u8;3];
    loop{
        thread::sleep(time::Duration::from_millis(100));
        match input.read(&mut buf){
            Ok(_)=>{
                if buf == [0,0,0]{
                    continue;
                } 
                func(&buf);

            },
            Err(_)=>()
        }
    }
}

fn handle_strike(buf: &[u8;3]){
    if (buf[1], buf[2]) == (255,255)  {
        println!("Hit");
        return;

    } else if (buf[1], buf[2]) == (254,254) {
        println!("Miss");
        return;
        
    } else {

        println!("Message delivered successfully");
        let mut player_field=PLAYER_FIELD.lock().unwrap();
        let mut output = OUTPUT.lock().unwrap();
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
        let field = &mut PLAYER_FIELD.lock().unwrap();
        field.field[r as usize][c as usize]
    };
    let ot_callback = |(r,c):(i32,i32)|->u8{
        let field = &mut OPPONNENT_FIELD.lock().unwrap();
        field.field[r as usize][c as usize]
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
                    prep_window.hide();
                    match_window.show();
                    
                },
                CustomEvents::WindowClosed=>{
                    println!("Closed");

                    return;
                },
                CustomEvents::PlayerStrikes=>{
                    println!("Player strikes");
                    let mut output=OUTPUT.lock().unwrap();
                    output.write(&[30,5,5]);
                },
                CustomEvents::ConnectAsServer=>{
                    connect_as_server();
                },
                CustomEvents::ConnectAsClient=>{
                   connect_as_client();
                },
            }
        }
    };
    
    app.run().unwrap();
}
