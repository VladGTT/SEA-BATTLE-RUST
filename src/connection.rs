use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use fltk::app::Sender;
use std::thread::JoinHandle;

use crate::ui::GUIEvents;

const SOCKET: &str = "127.0.0.1:8888";











#[derive(Copy, Clone)]
pub struct Message {
    pub data: [u8; 2],
}

pub struct Connection {
    stream: TcpStream,
}
impl Clone for Connection {
    fn clone(&self) -> Self {
        Connection {
            stream: self.stream.try_clone().unwrap(),
        }
    }
}

// pub fn listen_for(&self,mes: Message)->JoinHandle<Result<(),()>>{
//     let mut stream=self.stream.try_clone().unwrap();
//     std::thread::spawn(move||{
//         let mut buf = [0 as u8;2];
//         loop{
//             std::thread::sleep(std::time::Duration::from_millis(100));
//             match stream.read(&mut buf){
//                 Ok(_)=>{
//                     // println!("Heard {:?}",buf);
//                     if mes.data == buf {
//                         return Ok(())
//                     }
//                     else{
//                         return Err(())
//                     }
//                 },
//                 Err(_)=>()
//             }
//         }
//     })
// }

impl Connection {
    pub fn connect_as_server(sender: Sender<GUIEvents>) -> Result<Self, ()> {
        match TcpListener::bind(SOCKET).unwrap().accept() {
            Ok((stream, _)) => {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));

                let mut str = stream.try_clone().unwrap();
                str.write(&[125, 125]).unwrap();

                Connection::keepalive(str, [125, 125], [126, 126],sender);
                Ok(Connection { stream: stream })
            }
            Err(_) => Err(()),
        }
    }
    pub fn connect_as_client(socket: &str,sender: Sender<GUIEvents>) -> Result<Self, ()> {
        match TcpStream::connect(socket) {
            Ok(stream) => {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));

                let str = stream.try_clone().unwrap();

                Connection::keepalive(str, [126, 126], [125, 125],sender);
                Ok(Connection { stream: stream })
            }
            Err(_) => Err(()),
        }
    }

    pub fn write(&self, data: Message) -> Result<(), ()> {
        match self.stream.try_clone().unwrap().write(&data.data) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    pub fn listen(&self) -> JoinHandle<Message> {
        let mut stream = self.stream.try_clone().unwrap();
        let mut buf = [0 as u8; 2];

        let handle = std::thread::spawn(move || loop {
            match stream.peek(&mut buf) {
                Ok(_) => {
                    if buf == [125, 125] || buf == [126, 126] {
                        continue;
                    }
                    stream.read(&mut buf);
                    return Message { data: buf };
                }
                Err(_) => (),
            }
        });

        handle
    }

    fn keepalive(mut str: TcpStream, ping: [u8; 2], pong: [u8; 2],sender:Sender<GUIEvents>) {
        std::thread::spawn(move || {
            let mut is_break_notification_sent = false;
            let mut is_restore_notification_sent = true;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(30));
                let mut buf = [0 as u8; 2];
                match str.peek(&mut buf) {
                    Ok(_) => {
                        if buf != pong {
                            continue;
                        }
                        str.read(&mut buf).unwrap();
                        str.write(&ping).unwrap();

                        if !is_restore_notification_sent {
                            //send notification
                            sender.send(GUIEvents::ConnectionReistablished);
                            // println!("Connection reestablished");
                            is_restore_notification_sent = true;
                        }
                        if is_break_notification_sent {
                            is_break_notification_sent = false;
                        }
                    }
                    Err(_) => {
                        if str.write(&ping).is_ok() {
                            if !is_break_notification_sent {
                                //send notification
                                sender.send(GUIEvents::ConnectionDropped);
                                // println!("Connection lost");
                                is_break_notification_sent = true;
                            }
                            if is_restore_notification_sent {
                                is_restore_notification_sent = false;
                            }
                        } else {
                            //send signal
                            sender.send(GUIEvents::ConnectionDisconnected);
                            // println!("Opponent disconnected");
                            return;
                        }
                    }
                }
            }
        });
    }
}
