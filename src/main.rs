use fltk::{
    enums::{ Color, Event, Font},
    prelude::{WidgetExt, *},
    *, app::Sender, table::Table,
};
use once_cell::sync::Lazy;
use std::sync::{Mutex,MutexGuard};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::{thread,time};

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

struct PlayField{
    field:[[u8; 10]; 10],
    numb_4deck:i32,
    numb_3deck:i32,
    numb_2deck:i32,
    numb_1deck:i32,
}


impl Default for PlayField{
    fn default() -> Self {
        PlayField { 
            field: [[0 as u8; 10]; 10],
            numb_4deck:0,
            numb_3deck:0,
            numb_2deck:0,
            numb_1deck:0
         }
    }
}

impl PlayField {
    fn reset(&mut self){
        for i in 0..10{
            for j in 0..10{
                self.field[i][j]=0;
            } 
        }
        self.numb_4deck=0;
        self.numb_3deck=0;
        self.numb_2deck=0;
        self.numb_1deck=0;    
    }
    fn get_ship_numb(&self)->(i32,i32,i32,i32){
        (self.numb_1deck,self.numb_2deck,self.numb_3deck,self.numb_4deck)
    }

    fn place_ship(&mut self,selection:(i32,i32,i32,i32))->Result<(),()>{
        let deck_num = match Self::check_ship_deck(selection) {
            Ok(res) => res,
            Err(_) => {
                return Err(());
            }
        };

        match deck_num {
            4=>{
                if self.numb_4deck<MAX_4DECK{
                   self.set_ship_helper(selection)?; 
                   self.numb_4deck+=1;
                }
            },
            3=>{
                if self.numb_3deck<MAX_3DECK{
                   self.set_ship_helper(selection)?;
                   self.numb_3deck+=1;
                }
            },
            2=>{
                if self.numb_2deck<MAX_2DECK{
                   self.set_ship_helper(selection)?; 
                   self.numb_2deck+=1;
                }
            },
            1=>{
                if self.numb_1deck<MAX_1DECK{
                   self.set_ship_helper(selection)?; 
                   self.numb_1deck+=1; 
                }
            },
            _=>(),
        };

        Ok(())
    }
    fn check_ship_deck(selection: (i32, i32, i32, i32)) -> Result<i32, ()> {
        let height = selection.2 - selection.0;
        let width = selection.3 - selection.1;
    
        if height == 0 {
            return Ok(width + 1);
        }
        if width == 0 {
            return Ok(height + 1);
        }
        Err(())
    }
    fn check_surroundings_and_intersection(&mut self, selection: (i32, i32, i32, i32)) -> bool {
        let (mut rt, mut cl, mut rb, mut cr) = (
            selection.0 - 1,
            selection.1 - 1,
            selection.2 + 1,
            selection.3 + 1,
        );
    
        if rt < 0 {
            rt = 0;
        }
        if cl < 0 {
            cl = 0
        }
        if rb >= 10 {
            rb = 9;
        }
        if cr >= 10 {
            cr = 9;
        }
    
        for i in rt..rb + 1 {
            for j in cl..cr + 1 {
                if self.field[i as usize][j as usize] == 1 {
                    return true;
                }
            }
        }
        false
    }
    
    fn set_ship_helper(&mut self, selection: (i32, i32, i32, i32)) -> Result<(), ()> {
        let is_hor = selection.0 == selection.2;
        let is_ver = selection.1 == selection.3;
    
        if (!((is_hor) | (is_ver))) || self.check_surroundings_and_intersection(selection) {
            return Err(());
        }
    
        if is_hor {
            for i in selection.1..selection.3 + 1 {
                self.field[selection.0 as usize][i as usize] = 1;
            }
        }
        if is_ver {
            for i in selection.0..selection.2 + 1 {
                self.field[i as usize][selection.1 as usize] = 1;
            }
        }
        Ok(())
    }

    fn strike(&mut self, position:(u8,u8))->Result<bool,()>{
        let (x,y) = position;
        let val = self.field[x as usize][y as usize]; 

        match val {
            0=>{
                self.field[x as usize][y as usize]=2;
                return Ok(false);
            }
            1=>{
                self.field[x as usize][y as usize]=3;
                return Ok(true);
            }
            _=> return Err(())
        }
    }
}


struct PrepWindow{
    group: group::Group,

    table: table::Table,

    label_4deck: frame::Frame,
    label_3deck: frame::Frame,
    label_2deck: frame::Frame,
    label_1deck: frame::Frame,
}

impl PrepWindow{
    pub fn new(sender:Sender<CustomEvents>)->Self{
        let mut group=group::Group::new(0,0,800,600,None);
    
    
        let y_pos=50;
    
    
        let mut reset_btn = button::Button::default()
            .with_pos(480, y_pos+250)
            .with_size(100, 50)
            .with_label("Reset");
    
        reset_btn.set_callback(move|_|sender.send(CustomEvents::ResetField));
    
        let mut ready_btn = button::Button::default()
            .with_pos(630, y_pos+250)
            .with_size(100, 50)
            .with_label("Ready");
        ready_btn.set_callback( move|_|sender.send(CustomEvents::Ready));
    
    
    
        let mut label_4deck = frame::Frame::default()
            .with_pos(600, y_pos)
            .with_label(&format!("Ships with 4 decks remained: {}",MAX_4DECK));
    
        label_4deck.set_label_font(Font::Helvetica);
        label_4deck.set_label_size(16);
    
    
    
        let mut label_3deck = frame::Frame::default()
            .with_pos(600, y_pos+50)
            .with_label(&format!("Ships with 3 decks remained: {}",MAX_3DECK));
    
        label_3deck.set_label_font(Font::Helvetica);
        label_3deck.set_label_size(16);
    
    
    
        let mut label_2deck = frame::Frame::default()
            .with_pos(600, y_pos+100)
            .with_label(&format!("Ships with 2 decks remained: {}",MAX_2DECK));
    
        label_2deck.set_label_font(Font::Helvetica);
        label_2deck.set_label_size(16);
    
    
    
        let mut label_1deck = frame::Frame::default()
            .with_pos(600, y_pos+150)
            .with_label(&format!("Ships with 1 decks remained: {}",MAX_1DECK));
    
        label_1deck.set_label_font(Font::Helvetica);
        label_1deck.set_label_size(16);
    
        let mut table = table::Table::default().with_size(427, 427);

        table.set_rows(10);
        table.set_row_header(true);
        table.set_row_resize(true);
        table.set_cols(10);
        table.set_col_header(true);
        table.set_col_width_all(40);
        table.set_row_height_all(40);
        table.set_row_header_width(25);
        table.set_col_header_height(25);
        table.end();
    
        table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
            table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
            table::TableContext::ColHeader => draw_header(&format!("{}", (col + 65) as u8 as char), x, y, w, h), 
            table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
            table::TableContext::Cell => {
                let field = &mut PLAYER_FIELD.lock().unwrap();
                draw_data(x,y,w,h,t.is_selected(row, col),field.field[row as usize][col as usize]);
            }
            _ => (),
        });
        
        table.handle(move|_, event| match event{
            Event::Released => {
                sender.send(CustomEvents::ShipPlaced);
                true
            }
            _ => false,
        });

        group.add(&table);
        group.add(&label_4deck);
        group.add(&label_3deck);
        group.add(&label_2deck);
        group.add(&label_1deck);
        group.add(&reset_btn);
        group.add(&ready_btn);

        group.end();

        PrepWindow { 
                group: group,
                table: table,
                label_4deck:label_4deck,
                label_3deck:label_3deck,
                label_2deck:label_2deck,
                label_1deck:label_1deck 
            }

    }


    pub fn reset(&mut self){
        let mut field = PLAYER_FIELD.lock().unwrap();

        field.reset();

        let (n_1decks ,n_2decks,n_3decks,n_4decks)=field.get_ship_numb();

        self.label_1deck.set_label(&format!("Ships with {} decks remained: {}",1,MAX_1DECK-n_1decks));
        self.label_1deck.set_label_color(Color::Black);

        self.label_2deck.set_label(&format!("Ships with {} decks remained: {}",2,MAX_2DECK-n_2decks));
        self.label_2deck.set_label_color(Color::Black);

        self.label_3deck.set_label(&format!("Ships with {} decks remained: {}",3,MAX_3DECK-n_3decks));
        self.label_3deck.set_label_color(Color::Black);

        self.label_4deck.set_label(&format!("Ships with {} decks remained: {}",4,MAX_4DECK-n_4decks));
        self.label_4deck.set_label_color(Color::Black);
        self.group.redraw();
    }

    pub fn place_ship(&mut self){
        let (rt, cl, rb, cr) = self.table.get_selection();

        let field = &mut PLAYER_FIELD.lock().unwrap();

        let _ = field.place_ship((rt, cl, rb, cr));
        
        let (n_1decks ,n_2decks,n_3decks,n_4decks)=field.get_ship_numb();


        let update_labels=|label: &mut frame::Frame,remaining_ships: i32,deck_number:i32|{
            label.set_label(&format!("Ships with {} decks remained: {}",deck_number,remaining_ships));
            if remaining_ships == 0 {
                label.set_label_color(COLOR);
            }
        };

        update_labels(&mut self.label_4deck,MAX_4DECK-n_4decks,4);
        update_labels(&mut self.label_3deck,MAX_3DECK-n_3decks,3);
        update_labels(&mut self.label_2deck,MAX_2DECK-n_2decks,2);
        update_labels(&mut self.label_1deck,MAX_1DECK-n_1decks,1);

        self.group.redraw();
    }

    
    pub fn hide(&mut self){
        self.group.hide();
    }
    pub fn show(&mut self){
        self.group.show();
    }
}


fn draw_header (txt: &str, x: i32, y: i32, w: i32, h: i32) {
    draw::push_clip(x, y, w, h);
    draw::draw_box(enums::FrameType::ThinUpBox,x,y,w,h,enums::Color::FrameDefault);
    draw::set_draw_color(enums::Color::Black);
    draw::set_font(enums::Font::Helvetica, 14);
    draw::draw_text2(txt, x, y, w, h, enums::Align::Center);
    draw::pop_clip();
}
fn draw_data(x: i32, y: i32, w: i32, h: i32, selected: bool, value: u8) {
    draw::push_clip(x, y, w, h);

    if selected {
        draw::set_draw_color(enums::Color::from_u32(0x00D3_D3D3));
    } else {
        draw::set_draw_color(enums::Color::White);
    }
    match value{
        1=>draw::set_draw_color(enums::Color::Green),
        2=>draw::set_draw_color(enums::Color::Blue),
        3=>draw::set_draw_color(enums::Color::Red),
        _=>()
    }
    draw::draw_rectf(x, y, w, h);
    draw::set_draw_color(enums::Color::Gray0);
    draw::draw_rect(x, y, w, h);
    draw::pop_clip();
}

struct MatchWindow{
    group: group::Group
}

impl MatchWindow {
    fn new(sender:Sender<CustomEvents>) -> Self {
        let mut group=group::Group::new(0,0,800,600,None);

        let mut player_table = table::Table::default().with_size(427, 427);

        player_table.set_rows(10);
        player_table.set_row_header(true);
        player_table.set_row_resize(true);
        player_table.set_cols(10);
        player_table.set_col_header(true);
        player_table.set_col_width_all(40);
        player_table.set_row_height_all(40);
        player_table.set_row_header_width(25);
        player_table.set_col_header_height(25);
        player_table.end();
    
        player_table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
            table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
            table::TableContext::ColHeader => draw_header(&format!("{}", (col + 65) as u8 as char), x, y, w, h), 
            table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
            table::TableContext::Cell => {
                let field = &mut PLAYER_FIELD.lock().unwrap();
                draw_data(x,y,w,h,t.is_selected(row, col),field.field[row as usize][col as usize]);
            }
            _ => (),
        });
        

        group.add(&player_table);

        let mut opponent_table = table::Table::default().with_size(427, 427);

        opponent_table.set_pos(450, 0);
        opponent_table.set_rows(10);
        opponent_table.set_row_header(true);
        opponent_table.set_row_resize(true);
        opponent_table.set_cols(10);
        opponent_table.set_col_header(true);
        opponent_table.set_col_width_all(40);
        opponent_table.set_row_height_all(40);
        opponent_table.set_row_header_width(25);
        opponent_table.set_col_header_height(25);
        opponent_table.end();
    
        opponent_table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
            table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
            table::TableContext::ColHeader => draw_header(&format!("{}", (col + 65) as u8 as char), x, y, w, h), 
            table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
            table::TableContext::Cell => {
                let field = &mut OPPONNENT_FIELD.lock().unwrap();
                draw_data(x,y,w,h,t.is_selected(row, col),field.field[row as usize][col as usize]);
            }
            _ => (),
        });
        
        opponent_table.handle(move|_, event| match event{
            Event::Released => {
                sender.send(CustomEvents::PlayerStrikes);
                true
            }
            _ => false,
        });

        group.add(&player_table);
        group.add(&opponent_table);
        MatchWindow { group: group }
    }
    pub fn hide(&mut self){
        self.group.hide();
    }
    pub fn show(&mut self){
        self.group.show();
    }
}


const MAX_4DECK: i32 = 1;
const MAX_3DECK: i32 = 2;
const MAX_2DECK: i32 = 3;
const MAX_1DECK: i32 = 4;

const COLOR:Color=Color::DarkRed;

static PLAYER_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(PlayField::default()));
static OPPONNENT_FIELD: Lazy<Mutex<PlayField>> = Lazy::new(|| Mutex::new(PlayField::default()));
static MATCH: Lazy<Mutex<Match>> = Lazy::new(|| Mutex::new(Match::default()));

struct Match{
    is_started:bool,
    move_numb:Option<i32>,
    last_strike_coords:Option<(u8,u8)>
}

impl Match{
    fn default()->Self{
        Match { is_started: false, move_numb: None, last_strike_coords: None }
    }
}


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


const SOCKET_INPUT: &str = "localhost:8888"; 
const SOCKET_OUTPUT: &str = "localhost:8889"; 



static INPUT: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));
static OUTPUT: Lazy<Mutex<Connection>> = Lazy::new(|| Mutex::new(Connection::default()));

fn listen_input(input: &mut MutexGuard<'_, Connection>, func: &dyn Fn(&[u8;3])){
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

    
    let mut wind = window::Window::default().with_size(800, 600);
    wind.set_label("SEA BATTLE");

    let mut prep_window = PrepWindow::new(s);

    let mut match_window = MatchWindow::new(s);
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
                CustomEvents::ShipPlaced => prep_window.place_ship(),
                CustomEvents::ResetField=>prep_window.reset(),
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
                    

                    thread::spawn(move ||{
                        let mut input = INPUT.lock().unwrap();
                        match TcpListener::bind(SOCKET_INPUT).unwrap().accept(){
                            Ok((stream,_)) =>{
                                input.connect(stream);
                                println!("Input connection established");
                            },
                            Err(_) => println!("Connection input error")
                        }

                        listen_input(&mut input,&handle_strike);
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

                },
                CustomEvents::ConnectAsClient=>{
                    
                    thread::spawn(move ||{
                        let mut input = INPUT.lock().unwrap();
                        match TcpStream::connect(SOCKET_OUTPUT){
                            Ok(stream) =>{
                                input.connect(stream);
                                println!("Input connection established");
                            },
                            Err(_) => println!("Connection input error")
                        }

                        listen_input(&mut input,&handle_strike);
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

                },
            }
        }
    };
    
    app.run().unwrap();
}
