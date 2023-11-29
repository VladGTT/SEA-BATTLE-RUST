use crate::play_field::{PlayField, Field};




pub struct Battle{
    pub my_move: bool,

    pub player_field: PlayField,
    pub opponent_field: PlayField,
    pub last_coords: (i32,i32)
}

impl Default for Battle{
    fn default() -> Self {
        Battle { my_move: false,
            player_field: PlayField::new_player_field(),
            opponent_field: PlayField::new_opponent_field(),
            last_coords: (255,255)
        }
    }
}

