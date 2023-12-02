use std::net::{TcpStream,TcpListener};
use std::io::{Read, Write};
// use fltk::app::Sender;

const SOCKET: &str = "127.0.0.1:8888";

#[derive(Copy,Clone)]
pub struct Message{
    pub data: [u8;2]
}

pub struct Connection{
    stream:TcpStream,
}
impl Clone for Connection{
    fn clone(&self) -> Self {
        Connection { stream: self.stream.try_clone().unwrap() }
    }
}


impl Connection{
    pub fn connect_as_server()->Result<Self,()>{
        match TcpListener::bind(SOCKET).unwrap().accept(){
            Ok((stream,_)) =>{
                
                // Self::listen(self.stream.as_ref().unwrap(), sender);
                Ok(Connection { stream: stream})
            },
            Err(_) => Err(())
        }    

    }
    pub fn connect_as_client(socket: &str)->Result<Self,()>{
        match TcpStream::connect(socket){
            Ok(stream) =>{
                // Self::listen(self.stream.as_ref().unwrap(),sender);
                Ok(Connection { stream: stream})

            },
            Err(_) => Err(())
        }    
    }
    pub fn write(&self,data: Message)->Result<(),()>{
        match self.stream.try_clone().unwrap().write(&data.data){
            Ok(_)=>Ok(()),
            Err(_)=>Err(())
        }
    }
    
    pub async fn listen_for(&self,mes :&Message)->Result<(),()>{
        let mut stream=self.stream.try_clone().unwrap();
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
        let mut stream=self.stream.try_clone().unwrap();
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





