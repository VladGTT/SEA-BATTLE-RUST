#[derive(Clone,Copy)]
pub enum BattleResults{
    PlayerWon,
    PlayerLost,
}
#[derive(Clone,Copy)]
pub struct BattleStatistics{
    pub player_shots_hit: i32,
    pub player_shots_fired: i32,
    pub opponent_shots_hit: i32,
    pub opponent_shots_fired: i32,
    
    pub player_ships_destroed: (i32,i32,i32,i32), 
    pub opponent_ships_destroed: (i32,i32,i32,i32), 
    pub results: Option<BattleResults>
}

impl BattleStatistics{
    pub fn to_table(&self)->Vec<Vec<String>>{
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


        let player_results = vec![
            {if let BattleResults::PlayerWon=self.results.as_ref().unwrap(){"You"}else{""}}.to_string(),
            (if self.player_shots_fired!=0 {self.player_shots_hit/self.player_shots_fired}else{1}).to_string(),
            self.opponent_ships_destroed.3.to_string(),
            self.opponent_ships_destroed.2.to_string(),
            self.opponent_ships_destroed.1.to_string(),
            self.opponent_ships_destroed.0.to_string()
        ];

        retval.push(player_results);

        let opponent_results = vec![
            {if let BattleResults::PlayerLost=self.results.as_ref().unwrap(){"Opponent"}else{""}}.to_string(),
            (if self.opponent_shots_fired!=0 {self.opponent_shots_hit/self.opponent_shots_fired}else{1}).to_string(),
            self.player_ships_destroed.3.to_string(),
            self.player_ships_destroed.2.to_string(),
            self.player_ships_destroed.1.to_string(),
            self.player_ships_destroed.0.to_string()
        ];

        retval.push(opponent_results);

        retval
    }
}
