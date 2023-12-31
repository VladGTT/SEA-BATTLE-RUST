use fltk::{
    prelude::{WidgetExt, *},
    *, table::Table, button::Button, group::Group,
};
use crate::stats::{BattleStatistics,PlayersRating};
use std::sync::mpsc::Sender;


pub enum BattleResultsEvents{
    ButtonPressed
}
pub struct BattleResultWindow{
    pub group: Group,
    table: Table,
    button: Button
}

impl BattleResultWindow{
    pub fn set_handler(&mut self,sender:Sender<BattleResultsEvents>){
        self.button.set_callback(move|_|sender.send(BattleResultsEvents::ButtonPressed).unwrap());
    }


    pub fn show(&mut self){
        self.group.show();
        
    }

    pub fn hide(&mut self){
        self.group.hide();
    }
    pub fn new()->Self{
        let mut group=group::Group::new(0,0,800,600,None);


        let btn = button::Button::default()
            .with_pos(700, 0)
            .with_size(100, 50)
            .with_label("Ok");



        let mut table = table::Table::default().with_pos(0, 0).with_size(700, 300);

        table.set_rows(2);
        table.set_row_resize(true);
        table.set_cols(6);
        table.set_col_header(true);
        table.set_col_width_all(110);
        table.set_row_height_all(50);
        table.set_col_header_height(25);
        table.end();

        group.add(&table);
        group.add(&btn);

        group.end();

        BattleResultWindow {group: group, table: table, button: btn }
    }
    pub fn draw(&mut self,stats: (&BattleStatistics,&BattleStatistics),rating:&PlayersRating){

        let data=BattleStatistics::to_table(stats.0,stats.1,rating);

        self.table.draw_cell(move |_,cont,row,col,x,y,width,height|{
            match cont{
                table::TableContext::StartPage=>{draw::set_font(enums::Font::Helvetica, 14)},
                table::TableContext::ColHeader=>{
                    draw::push_clip(x, y, width, height);
                    draw::draw_box(enums::FrameType::ThinUpBox,x,y,width,height,enums::Color::FrameDefault);
                    draw::set_draw_color(enums::Color::Black);
                    draw::set_font(enums::Font::Helvetica, 14);
                    draw::draw_text2(&format!("{}",data[0 as usize][col as usize]), x, y, width, height, enums::Align::Center);
                    draw::pop_clip();
                },
                table::TableContext::Cell=>{
                    draw::push_clip(x, y, width, height);
    
                    draw::set_draw_color(enums::Color::White);
                    
                    draw::draw_rectf(x, y, width, height);
    
                    draw::set_draw_color(enums::Color::Gray0);
    
                    draw::draw_rect(x, y, width, height);

                    draw::draw_text2(&format!("{}",data[(row+1) as usize][col as usize]), x, y, width, height, enums::Align::Center);
    
                    draw::pop_clip();},
                _=>()
            }
        });

        self.group.redraw();
    }

}