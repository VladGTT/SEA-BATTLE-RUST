use fltk::{
    enums::{ Color, Event, Font},
    prelude::{WidgetExt, *},
    *, app::Sender, table::Table, frame::Frame,
};

use crate::{draw_table::{draw_data,draw_header}, play_field};
use crate::play_field::PlayField;
use crate::{MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK};

#[derive(Copy,Clone)]
pub enum BattleWindowEvents{
    Strike((i32,i32)),
}

pub struct BattleWindow{
    pub group: group::Group,
    player_field: Table,
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
        self.group.redraw();

    }
    pub fn set_handler(&mut self,sender:Sender<BattleWindowEvents>){

        self.opponent_field.handle(move|obj, event| match event{
            Event::Released => {
                let coords = obj.get_selection();
                sender.send(BattleWindowEvents::Strike((coords.0,coords.1)));
                true
            }
            _ => false,
        });
        
    }

    pub fn new()->Self{
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

        group.add(&player_table);
        group.add(&opponent_table);

        
        BattleWindow { group: group, player_field: player_table, opponent_field: opponent_table }
    }
}