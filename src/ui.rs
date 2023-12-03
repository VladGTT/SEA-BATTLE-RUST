
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

    Quit
}

pub fn render_gui(
    arg:
        fn(AppSender<GUIEvents>,
            Receiver<BattlePreparationEvents>,
            Receiver<BattleWindowEvents>,
            Receiver<BattleResultsEvents>,
            Receiver<ConnectionOptions>)->JoinHandle<()>)
        {
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

    
    


    arg(
        sender,
        battle_prep_reciever,
        battle_window_reciever,
        result_window_reciever,
        connection_window_reciever
    );
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

                GUIEvents::Quit=>{
                    app.quit();
                }
            }
        }
    }


    app.run().unwrap();
}
