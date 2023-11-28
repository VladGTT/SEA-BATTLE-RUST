use std::future::Future;
use std::net::{TcpStream,TcpListener};
use std::io::{Read, Write};
// use fltk::app::Sender;
use std::sync::mpsc::Sender;


#[derive(Copy,Clone)]
pub struct Message{
    pub data: [u8;2]
}

pub struct Connection{
    stream:Option<TcpStream>,
}
impl Default for Connection{
    fn default() -> Self {
        Connection { stream: None}
    }
}

impl Connection{
    pub fn connect_as_server(&mut self, socket: &str)->Result<(),()>{
        match TcpListener::bind(socket).unwrap().accept(){
            Ok((stream,_)) =>{
                self.stream=Some(stream);
                // Self::listen(self.stream.as_ref().unwrap(), sender);
                Ok(())
            },
            Err(_) => Err(())
        }    

    }
    pub fn connect_as_client(&mut self, socket: &str)->Result<(),()>{
        match TcpStream::connect(socket){
            Ok(stream) =>{
                self.stream=Some(stream);
                // Self::listen(self.stream.as_ref().unwrap(),sender);
                Ok(())
            },
            Err(_) => Err(())
        }    
    }
    pub fn write(&self,data: Message)->Result<(),()>{
        match self.stream.as_ref(){
            Some(str)=>{
                match str.try_clone().unwrap().write(&data.data){
                    Ok(_)=>Ok(()),
                    Err(_)=>Err(())
                }
            },
            None=>Err(())
        }
    }
    
    // async fn listen(str: &TcpStream)->Option<Message>{
    //     let mut stream=str.try_clone().unwrap();
    //     let mut buf = [0 as u8;2];
    //     loop{
    //         std::thread::sleep(std::time::Duration::from_millis(100));
    //         match stream.read(&mut buf){
    //             Ok(_)=>{
    //                     // println!("Heard {:?}",buf);
    //                     return Some(Message { data: buf });
    //             },
    //             Err(_)=>()  
    //         }
    //     }
    //     None
    // }
    pub async fn listen_for(&self,mes :&Message)->Result<(),()>{
        let mut stream=self.stream.as_ref().unwrap().try_clone().unwrap();
        let mut buf = [0 as u8;2];
        loop{
            std::thread::sleep(std::time::Duration::from_millis(100));
            match stream.read(&mut buf){
                Ok(_)=>{
                    // println!("Heard {:?}",buf);
                    if mes.data == buf {
                        return Ok(())
                    }
                    else{
                        return Err(())
                    }
                },
                Err(_)=>()  
            }
        }
    }
    pub async fn listen(&self)->Option<Message>{
        let mut stream=self.stream.as_ref().unwrap().try_clone().unwrap();
        let mut buf = [0 as u8;2];
        loop{
            std::thread::sleep(std::time::Duration::from_millis(100));
            match stream.read(&mut buf){
                Ok(_)=>{
                    // println!("Heard {:?}",buf);
                    return Some(Message { data: buf })
                },
                Err(_)=>return None  
            }
        }
    }
}





