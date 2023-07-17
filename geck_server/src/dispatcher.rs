use std::collections::HashMap;
use std::sync::{mpsc, Mutex, Arc};
use std::net::SocketAddr;
use std::error::Error;
use log::{debug, error, info};//{debug, error, warn, log_enabled, info, trace, Level};
use std::{io, thread, time::Duration};



pub struct Dispatcher {
    rx: mpsc::Receiver<(SocketAddr, Vec<u8>)>,
    tx: mpsc::Sender<(SocketAddr, Vec<u8>)>,
    book_of_clients: HashMap<SocketAddr, String>,
    new_conn_seq: Vec<u8>
}



impl Dispatcher {
    pub fn new(rx: mpsc::Receiver<(SocketAddr, Vec<u8>)>, tx: mpsc::Sender<(SocketAddr, Vec<u8>)>) -> Result<Dispatcher, Box<dyn Error>> {
        Ok(Dispatcher {
            rx,
            tx,
            book_of_clients:HashMap::new(),
            new_conn_seq: vec![0x02, 0x05, 0x83]
        })
    }

/*------------------------------------------------------------------------------------------------------------ */

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("Start data gathering/parsing");

        loop{
            let (addr, mut message) = self.rx.recv().expect("Parser mpsc stream is down");
            
            let name = self.book_of_clients.get(&addr);
            //2, 2, 83 as satrt bytes means a new user
            if message.len() == 0 && name.is_some(){

                info!("Removing client: {}", name.unwrap());
                self.book_of_clients.remove(&addr);
            }
            else if message.starts_with(&self.new_conn_seq) {
                if self.book_of_clients.get(&addr).is_none() {
                    let start = self.new_conn_seq.len();
                    
                    if let Ok(s) = String::from_utf8(message[start..].to_vec()){
                        info!("New client with name {}", s);
                        self.book_of_clients.insert(addr, s);
                    }
                }
            }
            else {
                for (client, name) in &self.book_of_clients {
                    if ! (*client == addr) {
                        let mut new_vec = name.as_bytes().to_vec();
                        new_vec.extend_from_slice(" : ".as_bytes());
                        new_vec.append(&mut message);
                        if self.tx.send((*client, new_vec)).is_err(){
                            error!("Error on parser transmit data");
                        }
                    }
                }
            }
        }
    }

/*------------------------------------------------------------------------------------------------------------ */

}


// pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
//     debug!("Start data gathering/parsing");

//     loop{
//         let (addr, mut message) = self.rx.recv().expect("Parser mpsc stream is down");

//             for (client, name) in &self.book_of_clients {
//                 let mut new_vec = name.as_bytes().to_vec();
//                 new_vec.append(message);
//             }
//         }
//     }
    