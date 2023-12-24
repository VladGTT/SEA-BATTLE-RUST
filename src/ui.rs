
use crate::game::GameEvents;
use crate::play_field::PlayField;
use crate::stats::{BattleStatistics,PlayersRating};

use fltk::dialog;
use fltk::{
    app::{self, Receiver as AppReciever, Sender as AppSender},
    enums,
    prelude::*,
    window
};
use std::{sync::mpsc::{Receiver, Sender}, thread::JoinHandle};

use crate::battle_prep_window::{BattlePrepWindow, BattlePreparationEvents};
use crate::battle_results_window::*;
use crate::battle_window::{BattleWindow, BattleWindowEvents};
use crate::connection_window::{ConnectionOptions,ConnectionWindow};

pub enum GUIEvents {
    RedrawBattleWindow(PlayField, PlayField),
    RedrawPreparationsWindow(PlayField),
    RedrawResultsWindow(BattleStatistics,BattleStatistics,PlayersRating),

    DisableBattleWindow,
    EnableBattleWindow,

    HideBattleWindow,
    ShowBattleWindow,

    HidePreparationsWindow,
    ShowPreparationsWindow,

    HideResultsWindow,
    ShowResultsWindow,

    ShowConnectionWindow,
    HideConnectionWindow,

    MarkWindowAsServer,
    MarkWindowAsClient,

    ShowGameResults(i32,i32),


    ConnectionDropped,
    ConnectionReistablished,
    ConnectionDisconnected,

    PlayerSurrendered,
    OpponentSurrendered,

    UpdatePlayerShipNumber((i32,i32,i32,i32)),
    UpdateOpponentShipNumber((i32,i32,i32,i32)),

    NotAllShipsPlaced,

    Quit
}
const DELAY: std::time::Duration = std::time::Duration::from_micros(100); 
pub fn render_gui(arg:fn(AppSender<GUIEvents>,Sender<GameEvents>,Receiver<GameEvents>)->JoinHandle<()>){
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = window::Window::default().with_size(800, 600);

    let (sender, reciever): (AppSender<GUIEvents>, AppReciever<GUIEvents>) = app::channel();
    let (battle_prep_sender, battle_prep_reciever): (
        Sender<BattlePreparationEvents>,
        Receiver<BattlePreparationEvents>,
    ) = std::sync::mpsc::channel();
    let (battle_window_sender, battle_window_reciever): (
        Sender<BattleWindowEvents>,
        Receiver<BattleWindowEvents>,
    ) = std::sync::mpsc::channel();
    let (result_window_sender, result_window_reciever): (
        Sender<BattleResultsEvents>,
        Receiver<BattleResultsEvents>,
    ) = std::sync::mpsc::channel();
    
    let (connection_window_sender, connection_window_reciever): (
        Sender<ConnectionOptions>,
        Receiver<ConnectionOptions>,
    ) = std::sync::mpsc::channel();
    

    
    
    
    let mut battle_window = BattleWindow::new();
    battle_window.set_handler(battle_window_sender);
    battle_window.hide();
    wind.add(&battle_window.group);
    
    let mut battle_prep_window = BattlePrepWindow::new();
    battle_prep_window.set_handler(battle_prep_sender);
    battle_prep_window.hide();    
    wind.add(&battle_prep_window.group);

    let mut battle_results_window = BattleResultWindow::new();
    battle_results_window.set_handler(result_window_sender);
    battle_results_window.hide();
    wind.add(&battle_results_window.group);

    let mut connection_window = ConnectionWindow::new();
    connection_window.set_handler(connection_window_sender);
    connection_window.hide();
    wind.add(&connection_window.flex);


    wind.make_resizable(true);

    {
        let s=sender.clone();
        wind.set_callback(move |_| {
            if app::event() == enums::Event::Close {
                s.send(GUIEvents::Quit);
            }
        });
        
    }

    let (game_events_sender, game_events_reciever): (
        Sender<GameEvents>,
        Receiver<GameEvents>,
    ) = std::sync::mpsc::channel();
   
    


    arg(sender.clone(),game_events_sender.clone(),game_events_reciever);

    wind.show();
   
    


    while app.wait() {
        if let Some(msg) = reciever.recv() {
            match msg {
                GUIEvents::HideBattleWindow => {
                    battle_window.hide();
                },
                GUIEvents::ShowBattleWindow => {
                    battle_window.show();
                },

                GUIEvents::HidePreparationsWindow => {
                    battle_prep_window.hide();
                },
                GUIEvents::ShowPreparationsWindow =>{
                    battle_prep_window.show();
                },

                GUIEvents::HideResultsWindow => {
                    battle_results_window.hide();
                }
                GUIEvents::ShowResultsWindow => {
                    battle_results_window.show();
                }

                GUIEvents::DisableBattleWindow => {
                    battle_window.opponent_field.deactivate();
                },
                GUIEvents::EnableBattleWindow => {
                    battle_window.opponent_field.activate();
                },

                GUIEvents::RedrawBattleWindow(player_field, opponent_field) => {
                    battle_window.draw(&player_field, &opponent_field);
                }

                GUIEvents::RedrawPreparationsWindow(field) => {
                    battle_prep_window.draw(&field);
                }

                GUIEvents::RedrawResultsWindow(table1,table2,rating) => {
                    battle_results_window.draw((&table1,&table2),&rating)
                }

                GUIEvents::ShowConnectionWindow=>{
                    connection_window.show();
                }

                GUIEvents::HideConnectionWindow=>{
                    connection_window.hide();
                }

                GUIEvents::MarkWindowAsClient=>{
                    wind.set_label("SEA-BATTLE-CLIENT");
                }
                
                GUIEvents::MarkWindowAsServer=>{
                    wind.set_label("SEA-BATTLE-SERVER");
                }

                GUIEvents::ShowGameResults(player_res,opponent_res)=>{
                    let mut txt = String::default();
                    if player_res==opponent_res{
                        txt = format!("Stalemate with results {}:{}",player_res,opponent_res);
                    }
                    
                    if player_res>opponent_res{
                        txt = format!("Winner {} with results {}:{}","You",player_res,opponent_res);
                    } else {
                        txt = format!("Winner {} with results {}:{}","Opponent",opponent_res,player_res);
                    }
                    
                    let _ = dialog::message_default(&txt);
                    app.quit();
                }

                GUIEvents::ConnectionDisconnected=>{
                    let _ = dialog::message_default("Opponnect disconnected");
                    app.quit()                    
                }

                GUIEvents::ConnectionDropped=>{
                    let _ = dialog::message_default("Connection dropped");
                    wind.deactivate();
                }
                
                GUIEvents::ConnectionReistablished=>{
                    let _ = dialog::message_default("Connection reistalished");
                    wind.activate();
                }



                GUIEvents::PlayerSurrendered=>{
                    let _ = dialog::message_default("You surrenderred, opponent won this battle");

                }
                GUIEvents::OpponentSurrendered=>{
                    let _ = dialog::message_default("Opponent surrenderred, you won this battle");
                    
                }

                GUIEvents::NotAllShipsPlaced=>{
                    let _ = dialog::message_default("Can't save scene not all ships are placed");
                }

                GUIEvents::UpdatePlayerShipNumber(numb)=>{

                }

                GUIEvents::UpdateOpponentShipNumber(numb)=>{

                }

                GUIEvents::Quit=>{
                    app.quit();
                }

            }
        }



        if let Ok(msg) = battle_prep_reciever.recv_timeout(DELAY){
            match msg {
                BattlePreparationEvents::Ready => {
                    game_events_sender.send(GameEvents::Ready);
                }
                BattlePreparationEvents::Reset => {
                    game_events_sender.send(GameEvents::Reset);
                    
                }
                BattlePreparationEvents::ShipPlaced(coords) => {
                    game_events_sender.send(GameEvents::ShipPlaced(coords));
                }


                BattlePreparationEvents::FieldLoaded=>{
                    let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseFile);
                    dialog.show();
                    game_events_sender.send(GameEvents::FieldLoaded(String::from(dialog.filename().to_str().unwrap())));
                }
                BattlePreparationEvents::FieldSaved=>{
                    let mut dialog = dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseSaveFile);
                    dialog.show();
                    game_events_sender.send(GameEvents::FieldSaved(String::from(dialog.filename().to_str().unwrap())));

                }
            }
        }



        if let Ok(msg) = battle_window_reciever.recv_timeout(DELAY){
            if let BattleWindowEvents::Strike(coords) = msg {
                game_events_sender.send(GameEvents::Strike(coords));
            }
            if let BattleWindowEvents::PlayerSurrendered = msg{
                sender.send(GUIEvents::PlayerSurrendered);
                game_events_sender.send(GameEvents::PlayerSurrendered);
            }
        }

        if let Ok(msg) = result_window_reciever.recv_timeout(DELAY){
            if let BattleResultsEvents::ButtonPressed = msg {
                game_events_sender.send(GameEvents::ToNextBattle);
            }
        }

        if let Ok(msg) = connection_window_reciever.recv_timeout(DELAY){
            game_events_sender.send(GameEvents::ConnectAs(msg));
        }


        
    }


    app.run().unwrap();
}
