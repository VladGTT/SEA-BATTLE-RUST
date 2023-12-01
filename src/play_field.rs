use crate::{MAX_1DECK, MAX_2DECK, MAX_3DECK, MAX_4DECK};

#[derive(Clone,Copy,PartialEq)]
pub enum PlayerFieldCell {
    Hit = 3,
    Missed = 2,
    Killed = 4,
    Bachground = 0,
    Ship = 1
}

#[derive(PartialEq, Eq)]
pub enum StrikeResponce {
    Hit,
    Miss,
    Kill,
    KilledLast
}




pub trait Field {
    fn new_player_field()->Self;
    fn new_opponent_field()->Self;
    fn reset(&mut self);
    fn get_ship_numb(&self) -> (i32, i32, i32, i32);
    fn check_ship_deck(selection: (i32, i32, i32, i32)) -> Option<i32>;
    fn mark_ship(
        &mut self,
        selection: (i32, i32, i32, i32),
        mark: PlayerFieldCell,
        predicate: fn(&Self,(i32, i32, i32, i32)) -> bool,
    ) -> Result<(), ()>;
}

pub trait GameField {
    fn strike_coords(&mut self, position: (u8, u8)) -> StrikeResponce;
    fn check_if_killed(&self, position: (u8, u8)) -> Option<(i32, i32, i32, i32)>;
    fn mark_as_hit(&mut self, position: (u8, u8));
    fn mark_as_miss(&mut self, position: (u8, u8));
    fn mark_as_kill(&mut self, coords: (i32, i32, i32, i32));
}

pub trait PrepField {
    fn check_surroundings_and_intersection(&self, selection: (i32, i32, i32, i32)) -> bool;
    fn place_ship(&mut self, selection: (i32, i32, i32, i32)) -> Result<(), ()>;
}

#[derive(Copy)]
pub struct PlayField {
    pub field: [[PlayerFieldCell; 10]; 10],
    numb_4deck: i32,
    numb_3deck: i32,
    numb_2deck: i32,
    numb_1deck: i32,
}

impl Clone for PlayField{
    fn clone(&self) -> Self {
        PlayField { field: self.field, 
            numb_4deck: self.numb_4deck, 
            numb_3deck: self.numb_3deck, 
            numb_2deck: self.numb_2deck, 
            numb_1deck: self.numb_1deck }
    }
    fn clone_from(&mut self, source: &Self) {
        self.field=source.field;
        self.numb_1deck=source.numb_1deck;
        self.numb_2deck=source.numb_2deck;
        self.numb_3deck=source.numb_3deck;
        self.numb_4deck=source.numb_4deck;
    }
}


impl Field for PlayField {
    fn new_player_field()->Self {
        PlayField {
            field: [[PlayerFieldCell::Bachground; 10]; 10],
            numb_4deck: 0,
            numb_3deck: 0,
            numb_2deck: 0,
            numb_1deck: 0,
        }
    }
    fn new_opponent_field()->Self {
        PlayField {
            field: [[PlayerFieldCell::Bachground; 10]; 10],
            numb_4deck: MAX_4DECK,
            numb_3deck: MAX_3DECK,
            numb_2deck: MAX_2DECK,
            numb_1deck: MAX_1DECK,
        }
    }
    fn reset(&mut self) {
        self.field = [[PlayerFieldCell::Bachground; 10]; 10];

        self.numb_4deck = 0;
        self.numb_3deck = 0;
        self.numb_2deck = 0;
        self.numb_1deck = 0;
    }

    fn check_ship_deck(selection: (i32, i32, i32, i32)) -> Option<i32> {
        let height = selection.2 - selection.0;
        let width = selection.3 - selection.1;

        if height == 0 {
            return Some(width + 1);
        }
        if width == 0 {
            return Some(height + 1);
        }
        None
    }

    fn get_ship_numb(&self) -> (i32, i32, i32, i32) {
        (
            self.numb_1deck,
            self.numb_2deck,
            self.numb_3deck,
            self.numb_4deck,
        )
    }
    fn mark_ship(
        &mut self,
        selection: (i32, i32, i32, i32),
        mark: PlayerFieldCell,
        predicate: fn(&Self,(i32, i32, i32, i32)) -> bool,
    ) -> Result<(), ()> {
        let is_hor = selection.0 == selection.2;
        let is_ver = selection.1 == selection.3;

        if (!((is_hor) | (is_ver))) || predicate(self,selection) {
            return Err(());
        }

        if is_hor {
            for i in selection.1..selection.3 + 1 {
                self.field[selection.0 as usize][i as usize] = mark;
            }
        }
        if is_ver {
            for i in selection.0..selection.2 + 1 {
                self.field[i as usize][selection.1 as usize] = mark;
            }
        }
        Ok(())
    }
}

impl PrepField for PlayField {
    fn check_surroundings_and_intersection(&self, selection: (i32, i32, i32, i32)) -> bool {
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
                if self.field[i as usize][j as usize] == PlayerFieldCell::Ship {
                    return true;
                }
            }
        }
        false
    }
    fn place_ship(&mut self, selection: (i32, i32, i32, i32)) -> Result<(), ()> {
        let deck_num = match Self::check_ship_deck(selection) {
            Some(res) => res,
            None => {
                return Err(());
            }
        };

        match deck_num {
            4 => {
                if self.numb_4deck < MAX_4DECK {
                    self.mark_ship(selection,PlayerFieldCell::Ship,Self::check_surroundings_and_intersection)?;
                    self.numb_4deck += 1;
                }
            }
            3 => {
                if self.numb_3deck < MAX_3DECK {
                    self.mark_ship(selection,PlayerFieldCell::Ship,Self::check_surroundings_and_intersection)?;

                    self.numb_3deck += 1;
                }
            }
            2 => {
                if self.numb_2deck < MAX_2DECK {
                    self.mark_ship(selection,PlayerFieldCell::Ship,Self::check_surroundings_and_intersection)?;
                    self.numb_2deck += 1;
                }
            }
            1 => {
                if self.numb_1deck < MAX_1DECK {
                    self.mark_ship(selection,PlayerFieldCell::Ship,Self::check_surroundings_and_intersection)?;
                    self.numb_1deck += 1;
                }
            }
            _ => (),
        };

        Ok(())
    }
}

impl GameField for PlayField {
    fn strike_coords(&mut self, position: (u8, u8)) -> StrikeResponce {
        match self.field[position.0 as usize][position.1 as usize]{
            PlayerFieldCell::Missed => {
                self.mark_as_miss(position);
                StrikeResponce::Miss
            },
            PlayerFieldCell::Ship => match self.check_if_killed(position) {
                Some(coords) => {
                    self.mark_as_kill(coords);
                    
                    if self.get_ship_numb() == (0,0,0,0){
                        return StrikeResponce::KilledLast
                    }
                    StrikeResponce::Kill
                },
                None => {
                    self.mark_as_hit(position);
                    StrikeResponce::Hit
                },
            },
            _ => StrikeResponce::Miss,
        }
    }
    fn mark_as_hit(&mut self, position: (u8, u8)) {
        let (x, y) = position;
        self.field[x as usize][y as usize] = PlayerFieldCell::Hit;
    }
    fn mark_as_kill(&mut self, coords: (i32, i32, i32, i32)) {
        _ = self.mark_ship(coords, PlayerFieldCell::Killed, |_,_|false);

        match Self::check_ship_deck(coords).unwrap() {
            1 => self.numb_1deck -= 1,
            2 => self.numb_2deck -= 1,
            3 => self.numb_3deck -= 1,
            4 => self.numb_4deck -= 1,
            _ => (),
        };
    }
    fn mark_as_miss(&mut self, position: (u8, u8)) {
        let (x, y) = position;
        self.field[x as usize][y as usize] = PlayerFieldCell::Missed;
    }
    fn check_if_killed(&self, position: (u8, u8)) -> Option<(i32, i32, i32, i32)> {
        let (x, y) = (position.0 as i32, position.1 as i32);

        let mut hit_coords = (x, y, x, y);

    
        if x != 0 {
            let mut i = 1;
            while x - i >= 0 {
                let data = self.field[(x - i) as usize][y as usize];

                if data == PlayerFieldCell::Ship {
                    return None;
                }
                if data != PlayerFieldCell::Hit {
                    break;
                }

                hit_coords.0 = x - i;
                i += 1;
            }
        }

        if y != 0 {
            let mut i = 1;
            while y - i >= 0 {
                let data = self.field[x as usize][(y - i) as usize];

                if data == PlayerFieldCell::Ship {
                    return None;
                }
                if data != PlayerFieldCell::Hit {
                    break;
                }

                hit_coords.1 = y - i;
                i += 1;
            }
        }

        if x != 9 {
            let mut i = 1;
            while x + i <= 9 {
                let data = self.field[(x + i) as usize][y as usize];

                if data == PlayerFieldCell::Ship {
                    return None;
                }
                if data != PlayerFieldCell::Hit  {
                    break;
                }
                hit_coords.2 = x + i;
                i += 1;
            }
        }
        if y != 9 {
            let mut i = 1;
            while y + i <= 9 {
                let data = self.field[x as usize][(y + i) as usize];

                if data == PlayerFieldCell::Ship  {
                    return None;
                }
                if data != PlayerFieldCell::Hit  {
                    break;
                }
                hit_coords.3 = y + i;
                i += 1;
            }
        }

        Some(hit_coords)
    }
}
