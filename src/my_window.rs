use fltk::{
    enums::{ Color, Event, Font},
    prelude::{WidgetExt, *},
    *, app::Sender, table::Table, frame::Frame,
};
use crate::CustomEvents;
use crate::{MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK};
// use crate::PLAYER_FIELD;
const COLOR:Color=Color::DarkRed;

pub trait Visible{
    fn hide(&mut self);
    fn show(&mut self);
}

pub trait PrepareWindow{
    fn new_prep_window(sender:Sender<CustomEvents>,player_table_callback: fn((i32,i32))->u8)->Self;
    fn reset(&mut self);
    fn place_ship(&mut self,func: fn((i32,i32,i32,i32))->(i32,i32,i32,i32));
}

pub trait MatchWindow{
    fn new_match_window(sender:Sender<CustomEvents>,player_table_callback: fn((i32,i32))->u8,opponent_table_callback: fn((i32,i32))->u8,strike_callback: fn((u8,u8)))->Self;
}

pub struct MyWindow{
    group: group::Group,
}

impl PrepareWindow for MyWindow{
    fn new_prep_window(sender:Sender<CustomEvents>,player_table_callback: fn((i32,i32))->u8)->Self{
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
                draw_data(x,y,w,h,t.is_selected(row, col),player_table_callback((row,col)));
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

        MyWindow {group: group}

    }

    fn reset(&mut self){

        let mut label_4deck = self.group.child(1).unwrap();
        let mut label_3deck = self.group.child(2).unwrap();
        let mut label_2deck = self.group.child(3).unwrap();
        let mut label_1deck = self.group.child(4).unwrap();

        label_1deck.set_label(&format!("Ships with {} decks remained: {}",1,MAX_1DECK));
        label_1deck.set_label_color(Color::Black);

        label_2deck.set_label(&format!("Ships with {} decks remained: {}",2,MAX_2DECK));
        label_2deck.set_label_color(Color::Black);

        label_3deck.set_label(&format!("Ships with {} decks remained: {}",3,MAX_3DECK));
        label_3deck.set_label_color(Color::Black);

        label_4deck.set_label(&format!("Ships with {} decks remained: {}",4,MAX_4DECK));
        label_4deck.set_label_color(Color::Black);
        
        self.group.redraw();
    }

    fn place_ship(&mut self, func: fn((i32,i32,i32,i32))->(i32,i32,i32,i32)){

        
        let table = Table::from_dyn_widget(&self.group.child(0).unwrap()).unwrap();

        let mut label_4deck = Frame::from_dyn_widget(&self.group.child(1).unwrap()).unwrap();
        let mut label_3deck = Frame::from_dyn_widget(&self.group.child(2).unwrap()).unwrap();
        let mut label_2deck = Frame::from_dyn_widget(&self.group.child(3).unwrap()).unwrap();
        let mut label_1deck = Frame::from_dyn_widget(&self.group.child(4).unwrap()).unwrap();

        
        let update_labels=|label: &mut frame::Frame,remaining_ships: i32,deck_number:i32|{
            label.set_label(&format!("Ships with {} decks remained: {}",deck_number,remaining_ships));
            if remaining_ships == 0 {
                label.set_label_color(COLOR);
            }
        };
        
        let (n_1decks ,n_2decks,n_3decks,n_4decks)=func(table.get_selection());
      
        update_labels(&mut label_4deck ,MAX_4DECK-n_4decks,4);
        update_labels(&mut label_3deck,MAX_3DECK-n_3decks,3);
        update_labels(&mut label_2deck,MAX_2DECK-n_2decks,2);
        update_labels(&mut label_1deck,MAX_1DECK-n_1decks,1);

        self.group.redraw();
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


impl MatchWindow for MyWindow {
    fn new_match_window(sender:Sender<CustomEvents>,player_table_callback: fn((i32,i32))->u8,opponent_table_callback: fn((i32,i32))->u8,strike_callback: fn((u8,u8)))-> Self {
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
                draw_data(x,y,w,h,t.is_selected(row, col),player_table_callback((row,col)));
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
                // let field = &mut OPPONNENT_FIELD.lock().unwrap();
                draw_data(x,y,w,h,t.is_selected(row, col),opponent_table_callback((row,col)));
            }
            _ => (),
        });
        
        opponent_table.handle(move|table, event| match event{
            Event::Released => {
                let coords = table.get_selection();
                strike_callback((coords.0 as u8, coords.1 as u8));

                sender.send(CustomEvents::PlayerStrikes);
                true
            }
            _ => false,
        });

        group.add(&player_table);
        group.add(&opponent_table);
        MyWindow { group: group }
    }
    
}

impl Visible for MyWindow{
    fn hide(&mut self){
        self.group.hide();
    }
    fn show(&mut self){
        self.group.show();
    }
}