#[derive(Clone,Copy)]
pub struct BattleStatistics{
    pub player_shots_hit: i32,
    pub player_shots_fired: i32,
    
    pub player_ships_destroed: (i32,i32,i32,i32), 
    pub player_won: Option<bool>
}

#[derive(Clone, Copy)]
pub struct PlayersRating{
    pub n_wins_player: i32,
    pub n_wins_opponent: i32
}

impl BattleStatistics{
    pub fn calc_ships_destroed(&mut self,n_ships:(i32,i32,i32,i32)){
        self.player_ships_destroed.0 -= n_ships.0;
        self.player_ships_destroed.1 -= n_ships.1;
        self.player_ships_destroed.2 -= n_ships.2;
        self.player_ships_destroed.3 -= n_ships.3;
    }


    pub fn to_table(player_stats:&Self,opponent_stats: &Self,rating: &PlayersRating)->Vec<Vec<String>>{
        let mut retval: Vec<Vec<String>> = Vec::default();
        let header = vec![
            "Winner".to_string(),
            "Accuracy".to_string(),
            "4-decks destroed".to_string(),
            "3-decks destroed".to_string(),
            "2-decks destroed".to_string(),
            "1-decks destroed".to_string()
        ];
        
        retval.push(header);

        if rating.n_wins_player>rating.n_wins_opponent{
            retval.push(player_stats.to_vector());
            retval.push(opponent_stats.to_vector());
        }else{
            retval.push(opponent_stats.to_vector());
            retval.push(player_stats.to_vector());
        }

        retval
    }

    pub fn to_vector(&self)->Vec<String>{
        vec![
            {if self.player_won.unwrap(){"You"}else{""}}.to_string(),
            (if self.player_shots_fired!=0 {format!("{:.2}",self.player_shots_hit as f64/self.player_shots_fired as f64)}else{"N/A".to_string()}),
            self.player_ships_destroed.3.to_string(),
            self.player_ships_destroed.2.to_string(),
            self.player_ships_destroed.1.to_string(),
            self.player_ships_destroed.0.to_string()
        ]
    }
}
