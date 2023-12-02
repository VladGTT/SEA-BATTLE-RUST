mod battle_prep_window;
mod battle_results_window;
mod battle_window;
mod connection;
mod draw_table;
mod play_field;
mod stats;
mod connection_window;
mod ui;
mod game;

use ui::render_gui;
use crate::game::handle_game;

fn main() {
    render_gui(handle_game);
}