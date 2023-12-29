use fltk::{prelude::*, *};
use std::sync::mpsc::Sender;

use std::{net::Ipv4Addr, str::FromStr};
#[derive(Debug,Clone, Copy)]
pub enum ConnectionOptions{
    ConnectAsServer(Ipv4Addr,u8),
    ConnectAsClient(Ipv4Addr),
}

pub struct ConnectionWindow{
    pub flex: group::Flex,
    btn_server: button::Button,
    btn_client: button::Button,
}

impl ConnectionWindow{
    pub fn hide(&mut self){
        self.flex.hide();
    }
    pub fn show(&mut self){
        self.flex.show();
    }

    pub fn new()->Self{

        let flex = group::Flex::default()
            .with_size(150, 100)
            .column()
            .center_of_parent();
        let btn_server = button::Button::default().with_label("Connect as server");
        let btn_client = button::Button::default().with_label("Connect as client");
        
        flex.end();
        ConnectionWindow { flex: flex,btn_server: btn_server, btn_client:btn_client }
    }
    pub fn set_handler(&mut self,s: Sender<ConnectionOptions>){
        let server=s.clone();
        self.btn_server.set_callback(move |_|{
            loop{
                let str_addr = match dialog::input_default("Input server socket", "127.0.0.1"){
                    Some(str)=>{
                        str
                    },
                    None=>continue
                };

                let addr=match Ipv4Addr::from_str(&str_addr){
                    Ok(val)=>val,
                    Err(_)=>continue
                };


                let str = match dialog::input_default("Input battle number", "1"){
                    Some(str)=>str,
                    None=>continue
                };

                let numb=match str.parse::<u8>(){
                    Ok(val)=>val,
                    Err(_)=>continue
                };

                _=server.send(ConnectionOptions::ConnectAsServer(addr,numb));
                break;
            }
        });

        self.btn_client.set_callback(move |_|{
            loop{
                let dialog = dialog::input_default("Input server socket", "127.0.0.1");

                let str = match dialog{
                    Some(str)=>str,
                    None=>continue
                };

                let addr=match Ipv4Addr::from_str(&str){
                    Ok(val)=>val,
                    Err(_)=>continue
                };

                _ = s.send(ConnectionOptions::ConnectAsClient(addr));
                break;
            }            
        });
    }
    
}