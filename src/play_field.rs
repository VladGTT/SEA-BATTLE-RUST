use crate::{MAX_1DECK,MAX_2DECK,MAX_3DECK,MAX_4DECK};


pub struct PlayField{
    pub field:[[u8; 10]; 10],
    numb_4deck:i32,
    numb_3deck:i32,
    numb_2deck:i32,
    numb_1deck:i32,
}


impl Default for PlayField{
    fn default() -> Self {
        PlayField { 
            field: [[0 as u8; 10]; 10],
            numb_4deck:0,
            numb_3deck:0,
            numb_2deck:0,
            numb_1deck:0
         }
    }
}

impl PlayField {
    pub fn reset(&mut self){
        for i in 0..10{
            for j in 0..10{
                self.field[i][j]=0;
            } 
        }
        self.numb_4deck=0;
        self.numb_3deck=0;
        self.numb_2deck=0;
        self.numb_1deck=0;    
    }


    pub fn get_ship_numb(&self)->(i32,i32,i32,i32){
        (self.numb_1deck,self.numb_2deck,self.numb_3deck,self.numb_4deck)
    }

    
    pub fn strike(&mut self, position:(u8,u8))->Result<bool,()>{
        let (x,y) = position;
        let val = self.field[x as usize][y as usize]; 

        match val {
            0=>{
                self.field[x as usize][y as usize]=2;
                return Ok(false);
            }
            1=>{
                self.field[x as usize][y as usize]=3;
                return Ok(true);
            }
            _=> return Err(())
        }
    }



    pub fn place_ship(play_field: &mut PlayField,selection:(i32,i32,i32,i32))->Result<(),()>{
        
        
        let check_ship_deck = |selection: (i32, i32, i32, i32)| -> Result<i32, ()> {
            let height = selection.2 - selection.0;
            let width = selection.3 - selection.1;
        
            if height == 0 {
                return Ok(width + 1);
            }
            if width == 0 {
                return Ok(height + 1);
            }
            Err(())
        };
        
        let check_surroundings_and_intersection = |selection: (i32, i32, i32, i32),field: &[[u8;10];10] |->bool{
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
                    if field[i as usize][j as usize] == 1 {
                        return true;
                    }
                }
            }
            false
        };
        
        
        let set_ship_helper = |selection: (i32, i32, i32, i32),field: &mut [[u8;10];10]| -> Result<(), ()> {
        
            let is_hor = selection.0 == selection.2;
            let is_ver = selection.1 == selection.3;
        
            if (!((is_hor) | (is_ver))) || check_surroundings_and_intersection(selection,field) {
                return Err(());
            }
        
            if is_hor {
                for i in selection.1..selection.3 + 1 {
                    field[selection.0 as usize][i as usize] = 1;
                }
            }
            if is_ver {
                for i in selection.0..selection.2 + 1 {
                    field[i as usize][selection.1 as usize] = 1;
                }
            }
            Ok(())
        };
        
        
        let deck_num = match check_ship_deck(selection) {
            Ok(res) => res,
            Err(_) => {
                return Err(());
            }
        };
    
        match deck_num {
            4=>{
                if play_field.numb_4deck<MAX_4DECK{
                   set_ship_helper(selection,&mut play_field.field)?; 
                   play_field.numb_4deck+=1;
                }
            },
            3=>{
                if play_field.numb_3deck<MAX_3DECK{
                   set_ship_helper(selection,&mut play_field.field)?; 
    
                   play_field.numb_3deck+=1;
                }
            },
            2=>{
                if play_field.numb_2deck<MAX_2DECK{
                   set_ship_helper(selection,&mut play_field.field)?; 
    
                   play_field.numb_2deck+=1;
                }
            },
            1=>{
                if play_field.numb_1deck<MAX_1DECK{
                   set_ship_helper(selection,&mut play_field.field)?; 
    
                   play_field.numb_1deck+=1; 
                }
            },
            _=>(),
        };
    
        Ok(())
    }


}