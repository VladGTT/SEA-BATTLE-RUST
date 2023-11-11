use std::net::{TcpStream,TcpListener};
use std::io::{Read, Write};
use fltk::app::Sender;

use crate::CustomEvents;
pub struct Connection{
    stream:Option<TcpStream>,
}
impl Default for Connection{
    fn default() -> Self {
        Connection { stream: None}
    }
}

impl Connection{
    pub fn connect_as_server(&mut self, socket: &str, func: fn(&[u8;3]),sender:Sender<CustomEvents>)->Result<(),()>{
        match TcpListener::bind(socket).unwrap().accept(){
            Ok((stream,_)) =>{
                self.stream=Some(stream);
                Self::listen(self.stream.as_ref().unwrap(), func,sender);
                Ok(())
            },
            Err(_) => Err(())
        }    

    }
    pub fn connect_as_client(&mut self, socket: &str, func: fn(&[u8;3]),sender:Sender<CustomEvents>)->Result<(),()>{
        match TcpStream::connect(socket){
            Ok(stream) =>{
                self.stream=Some(stream);
                Self::listen(self.stream.as_ref().unwrap(), func,sender);
                Ok(())
            },
            Err(_) => Err(())
        }    
    }
    pub fn write(&mut self,data: &[u8])->Result<(),()>{
        match self.stream{
            Some(ref mut str)=>{
                match str.write(data){
                    Ok(_)=>Ok(()),
                    Err(_)=>Err(())
                }
            },
            None=>Err(())
        }
    }
    
    // pub fn read(&mut self,buf: &mut [u8])->Result<(),()>{
    //     match self.stream{
    //         Some(ref mut str)=>{
    //             match str.read(buf){
    //                 Ok(_)=>Ok(()),
    //                 Err(_)=>Err(())
    //             }
    //         },
    //         None=>Err(())
    //     }
    // }

    fn listen(str: &TcpStream,func: fn(&[u8;3]),sender:Sender<CustomEvents>){
        let mut stream=str.try_clone().unwrap();
        std::thread::spawn(move ||{
            let mut buf = [0 as u8;3];
            loop{
                std::thread::sleep(std::time::Duration::from_millis(100));
                match stream.read(&mut buf){
                    Ok(_)=>{
                        if buf == [0,0,0]{continue;}
                        sender.send(CustomEvents::OpponentStrikes);
                        func(&buf); 
                    },
                    Err(_)=>()  
                }
            }

        });
    }
    
}





