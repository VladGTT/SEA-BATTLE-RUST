use crate::play_field::PrepField;

#[derive(Copy,Clone)]
enum BattlePreparationEvents{
    ShipPlaced(i32,i32,i32,i32),
    Ready,
    Reset
}

pub struct BattlePrepWindow{
    pub group: group::Group,
}


impl BattlePrepWindow{
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
            4=>draw::set_draw_color(enums::Color::DarkRed),
            _=>()
        }
        draw::draw_rectf(x, y, w, h);
        draw::set_draw_color(enums::Color::Gray0);
        draw::draw_rect(x, y, w, h);
        draw::pop_clip();
    }









    fn draw(data: PlayField){
        
    }
    fn new(sender:Sender<BattlePreparationEvents>,player_table_callback: fn((i32,i32))->u8)->Self{
        let mut group=group::Group::new(0,0,800,600,None);
    
    
        let y_pos=50;
    
    
        let mut reset_btn = button::Button::default()
            .with_pos(480, y_pos+250)
            .with_size(100, 50)
            .with_label("Reset");
    
        reset_btn.set_callback(move|_|sender.send(GameEvent { event_type: GameEventType::ResetField, data: None }));
    
        let mut ready_btn = button::Button::default()
            .with_pos(630, y_pos+250)
            .with_size(100, 50)
            .with_label("Ready");
        ready_btn.set_callback( move|_|sender.send(GameEvent { event_type: GameEventType::PlayerReady, data: None }));
    
    
    
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
        
        table.handle(move|obj, event| match event{
            Event::Released => {
                sender.send(BattlePreparationEvents::ShipPlaced(obj.get_selection()));
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

}