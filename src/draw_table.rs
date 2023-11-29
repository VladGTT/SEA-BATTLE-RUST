use fltk::*;

pub fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32) {
    draw::push_clip(x, y, w, h);
    draw::draw_box(enums::FrameType::ThinUpBox,x,y,w,h,enums::Color::FrameDefault);
    draw::set_draw_color(enums::Color::Black);
    draw::set_font(enums::Font::Helvetica, 14);
    draw::draw_text2(txt, x, y, w, h, enums::Align::Center);
    draw::pop_clip();
}
pub fn draw_data(x: i32, y: i32, w: i32, h: i32, selected: bool, value: u8) {
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