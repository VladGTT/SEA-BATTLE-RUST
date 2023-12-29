use fltk::{
    enums::{Event,Color,Font},
    prelude::{WidgetExt, *},
    *, table::Table, frame::Frame
};

use crate::{draw_table::{draw_data,draw_header}, play_field::PlayField};
use crate::play_field::Field;
use std::sync::mpsc::Sender;

use crate::game::{MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK};
use crate::battle_prep_window::{COLOR,DEFAULT_COLOR};

#[derive(Copy,Clone)]
pub enum BattleWindowEvents{
    Strike((i32,i32)),
    PlayerSurrendered
}

pub struct BattleWindow{
    pub group: group::Group,
    player_field: Table,
    button: button::Button,

    label_4deck: frame::Frame,
    label_3deck: frame::Frame,
    label_2deck: frame::Frame,
    label_1deck: frame::Frame,

    pub opponent_field: Table,
}

impl BattleWindow{
    pub fn show(&mut self){
        self.group.show();
    }

    pub fn hide(&mut self){
        self.group.hide();
    }

    pub fn draw(&mut self,player_field: &PlayField,opponent_field: &PlayField){
        let player_table = &mut self.player_field;
        let opponent_table = &mut self.opponent_field;
        let play_field_data=player_field.clone();
        let opponent_field_data=opponent_field.clone();
        player_table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
            table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
            table::TableContext::ColHeader => draw_header(&format!("{}", (col + 65) as u8 as char), x, y, w, h), 
            table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
            table::TableContext::Cell => {
                draw_data(x,y,w,h,t.is_selected(row, col),play_field_data.field[row as usize][col as usize] as u8);
            }
            _ => (),
        });
        opponent_table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
            table::TableContext::StartPage => draw::set_font(enums::Font::Helvetica, 14),
            table::TableContext::ColHeader => draw_header(&format!("{}", (col + 65) as u8 as char), x, y, w, h), 
            table::TableContext::RowHeader => draw_header(&format!("{}", row + 1), x, y, w, h),
            table::TableContext::Cell => {
                draw_data(x,y,w,h,t.is_selected(row, col),opponent_field_data.field[row as usize][col as usize] as u8);
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

        let (n_1decks ,n_2decks,n_3decks,n_4decks) = opponent_field.get_ship_numb();
      
        update_labels(&mut self.label_4deck ,n_4decks,4);
        update_labels(&mut self.label_3deck,n_3decks,3);
        update_labels(&mut self.label_2deck,n_2decks,2);
        update_labels(&mut self.label_1deck,n_1decks,1);




        self.group.redraw();

    }
    pub fn set_handler(&mut self,sender:Sender<BattleWindowEvents>){
        let s = sender.clone();

        self.button.handle(move|_, event| match event{
            Event::Released => {
                s.send(BattleWindowEvents::PlayerSurrendered).unwrap();
                true
            }
            _ => false,
        });


        self.opponent_field.handle(move|obj, event| match event{
            Event::Released => {
                let coords = obj.get_selection();
                sender.send(BattleWindowEvents::Strike((coords.0,coords.1))).unwrap();
                true
            }
            _ => false,
        });
        
    }

    pub fn new()->Self{
        let mut group=group::Group::default_fill();
        // let mut group=group::Flex::new(0,0,800,600,None);

        let btn = fltk::button::Button::new(500,500,100,50,"Surrender");
        let mut player_table = table::Table::default()
            // .with_pos(x, y)
            .with_size(427, 427);

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

        let mut opponent_table = table::Table::default()
        // .with_pos(x, y)
            .with_size(427, 427);
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




        let mut label_4deck = frame::Frame::default()
            .with_pos(150, 500)
            .with_label(&format!("Ships with 4 decks remained: {}",MAX_4DECK));

        label_4deck.set_label_font(Font::Helvetica);
        label_4deck.set_label_size(16);



        let mut label_3deck = frame::Frame::default()
            .with_pos(150, 520)
            .with_label(&format!("Ships with 3 decks remained: {}",MAX_3DECK));

        label_3deck.set_label_font(Font::Helvetica);
        label_3deck.set_label_size(16);



        let mut label_2deck = frame::Frame::default()
            .with_pos(150, 540)
            .with_label(&format!("Ships with 2 decks remained: {}",MAX_2DECK));

        label_2deck.set_label_font(Font::Helvetica);
        label_2deck.set_label_size(16);



        let mut label_1deck = frame::Frame::default()
            .with_pos(150, 560)
            .with_label(&format!("Ships with 1 decks remained: {}",MAX_1DECK));

        label_1deck.set_label_font(Font::Helvetica);
        label_1deck.set_label_size(16);


        group.add(&player_table);
        group.add(&opponent_table);

        group.end();

        BattleWindow { group: group, player_field: player_table, opponent_field: opponent_table, button: btn, 
            label_1deck:label_1deck,
            label_2deck:label_2deck,
            label_3deck:label_3deck,
            label_4deck:label_4deck
        }
    }
}