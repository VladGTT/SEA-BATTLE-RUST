use fltk::{
    enums::{ Color, Event, Font},
    prelude::{WidgetExt, *},
    *, table::Table, frame::Frame, button::Button, group::Group,
};

use crate::play_field::{Field, PlayField};
use crate::game::{MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK};
use crate::draw_table::{draw_data,draw_header};
use std::sync::mpsc::{Sender};

const COLOR:Color=Color::DarkRed;
const DEFAULT_COLOR:Color=Color::Black;

#[derive(Copy,Clone)]
pub enum BattlePreparationEvents{
    ShipPlaced((i32,i32,i32,i32)),
    Ready,
    Reset
}

pub struct BattlePrepWindow{
    pub group: Group,

    label_4deck:Frame,
    label_3deck:Frame,
    label_2deck:Frame,
    label_1deck:Frame,

    reset_btn: Button,
    ready_btn: Button,

    table: Table,
}


impl BattlePrepWindow{
    pub fn hide(&mut self){
        self.group.hide();
    }
    pub fn show(&mut self){
        self.group.show();

    }
    pub fn draw(&mut self,data: &PlayField){      
        
        let table = &mut self.table;
        
        let mut label_4deck = Frame::from_dyn_widget(&self.group.child(1).unwrap()).unwrap();
        let mut label_3deck = Frame::from_dyn_widget(&self.group.child(2).unwrap()).unwrap();
        let mut label_2deck = Frame::from_dyn_widget(&self.group.child(3).unwrap()).unwrap();
        let mut label_1deck = Frame::from_dyn_widget(&self.group.child(4).unwrap()).unwrap();

        // let mut label_4deck = &mut self.label_4deck;
        // let mut label_3deck = &mut self.label_3deck;
        // let mut label_2deck = &mut self.label_2deck;
        // let mut label_1deck = &mut self.label_1deck;

        let cloned_data=data.clone();
        table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
            table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
            table::TableContext::ColHeader => draw_header(&format!("{}", (col + 65) as u8 as char), x, y, w, h), 
            table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
            table::TableContext::Cell => {
                draw_data(x,y,w,h,t.is_selected(row, col),cloned_data.field[row as usize][col as usize] as u8);
            }
            _ => (),
        });


        let update_labels=|label: &mut frame::Frame,remaining_ships: i32,deck_number:i32|{
            label.set_label(&format!("Ships with {} decks remained: {}",deck_number,remaining_ships));
            if remaining_ships == 0 {
                label.set_label_color(COLOR);
            }
            else{
                label.set_label_color(DEFAULT_COLOR);
            }
        };
        
        let (n_1decks ,n_2decks,n_3decks,n_4decks)=data.get_ship_numb();
      
        update_labels(&mut label_4deck ,MAX_4DECK-n_4decks,4);
        update_labels(&mut label_3deck,MAX_3DECK-n_3decks,3);
        update_labels(&mut label_2deck,MAX_2DECK-n_2decks,2);
        update_labels(&mut label_1deck,MAX_1DECK-n_1decks,1);

        self.group.redraw();
    }

    pub fn set_handler(&mut self,sender:Sender<BattlePreparationEvents>){
        let table_sender=sender.clone();

        self.table.handle(move|obj, event| match event{
            Event::Released => {
                table_sender.send(BattlePreparationEvents::ShipPlaced(obj.get_selection()));
                true
            }
            _ => false,
        });


        let mut reset_btn = &mut self.reset_btn;
        let reset_sender=sender.clone();
        reset_btn.set_callback(move|_|reset_sender.send(BattlePreparationEvents::Reset).unwrap());


        let mut ready_btn = &mut self.ready_btn;
        let ready_sender=sender.clone();
        ready_btn.set_callback( move|_|ready_sender.send(BattlePreparationEvents::Ready).unwrap());
    }

    pub fn new()->Self{
        let mut group=group::Group::new(0,0,800,600,None);
    
    
        let y_pos=50;
    
    
        let mut reset_btn = button::Button::default()
            .with_pos(480, y_pos+250)
            .with_size(100, 50)
            .with_label("Reset");
    
    
        let mut ready_btn = button::Button::default()
            .with_pos(630, y_pos+250)
            .with_size(100, 50)
            .with_label("Ready");
    
    
    
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
    
        group.add(&table);
        group.add(&label_4deck);
        group.add(&label_3deck);
        group.add(&label_2deck);
        group.add(&label_1deck);
        group.add(&ready_btn);
        group.add(&reset_btn);
    
        group.end();


        BattlePrepWindow {
            group: group,
            table: table,
            label_4deck: label_4deck,
            label_3deck: label_3deck,
            label_2deck: label_2deck,
            label_1deck: label_1deck,
            ready_btn: ready_btn,
            reset_btn: reset_btn,
        }
    }

}