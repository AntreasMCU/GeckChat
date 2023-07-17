use std::error::Error;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::io::{Read, Write, ErrorKind, Error as ioError};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use log::error; //debug,error, warn, log_enabled, info, trace, Level};

pub struct TcpClient{
    connection_address: SocketAddr,
    from_client_rx: Option<mpsc::Receiver<Vec<u8>>>,
    from_client_tx: Option<mpsc::Sender<Vec<u8>>>,
    to_client_rx: Option<mpsc::Receiver<Vec<u8>>>,
    to_client_tx: Option<mpsc::Sender<Vec<u8>>>,
    stream: Option<Arc<Mutex<TcpStream>>>
}

impl TcpClient{
    pub fn new(addr: SocketAddr) -> Result<TcpClient, Box<dyn Error>>{

        Ok({
            TcpClient{
                connection_address: addr,
                from_client_rx: None,
                from_client_tx: None,
                to_client_rx: None,
                to_client_tx: None,
                stream: None
            }
        })
    }

    /*----------------------------------------------------------------------------- */

    pub fn start(&mut self) -> Result<(), Box<dyn Error>>{ //(mpsc::Receiver<Vec<u8>>, mpsc::Sender<Vec<u8>>

        let (from_client_tx , from_client_rx) = mpsc::channel::<Vec<u8>>();
        let (to_client_tx , to_client_rx) = mpsc::channel::<Vec<u8>>();

        self.from_client_rx = Some(from_client_rx).take();
        self.from_client_tx = Some(from_client_tx).clone();
        self.to_client_rx = Some(to_client_rx).take();
        self.to_client_tx = Some(to_client_tx).clone();

        let local_stream = match TcpStream::connect(self.connection_address){
            Ok(stream) => stream,
            Err(e) => {
                error!("Unable to connect to socket: {}. Error: {}", self.connection_address, e);
                return Err(Box::new(ioError::new(ErrorKind::Other, "Failed to connect to socket")));
            }
        };

        // self.stream = Some(Arc::new(Mutex::new(local_stream.try_clone().expect("Unable to clone TcpStream"))));
        let read_stream = local_stream.try_clone().expect("Unable to clone TcpStream");
        let read_from_client_tx = self.from_client_tx.clone().unwrap();//mpsc::Sender::clone(&self.from_client_tx.unwrap());
    
        let _ = thread::spawn( move || {
            Self::read_from_server(read_stream, read_from_client_tx);
        });

        let write_stream = local_stream.try_clone().expect("Unable to clone TcpStream");
        let write_to_client_rx = self.to_client_rx.take().expect("Unable to take to_client_rx");

        let _ = thread::spawn( move || {
            Self::write_to_server(write_stream, write_to_client_rx);
        });

        Ok(()) //(shared_receiver.unwrap(), shared_sender) 

    }

    /*----------------------------------------------------------------------------- */

    pub fn get_sender(&mut self) -> Result<mpsc::Sender<Vec<u8>>, Box<dyn Error>>{

        let shared_tx = self.to_client_tx.take().unwrap();
        Ok(shared_tx)
    }

    /*----------------------------------------------------------------------------- */

    pub fn get_receiver(&mut self) -> Result<mpsc::Receiver<Vec<u8>>, Box<dyn Error>>{

        let shared_receiver = self.from_client_rx.take().unwrap();
        return Ok(shared_receiver);
        
    }

    /*----------------------------------------------------------------------------- */

    fn read_from_server(mut stream: TcpStream, tx: mpsc::Sender<Vec<u8>>){
        loop{
            let mut buf = vec!(0; 1024);
            let _ = match stream.read(&mut buf){
                Ok(n) => {
                    if n == 0 {
                        // connection closed
                        error!("Connection closed from Server");
                        return;
                    }
                    else{
                        //reduce buffer size to minimum length. Data are copied so we need to be smaller in size
                        //truncate takes care of n==0 or n > buf.length
                        buf.truncate(n);
                        // info!("packet arrived. Data: {:?}", buf);
                        tx.send(buf).expect("Error on sending addr_packet to parser");
                    }
                },
                Err(e) => match e.kind() {
                    ErrorKind::ConnectionAborted => {
                        error!("ConnectionAborted from Server");
                        return;
                    },
                    ErrorKind::ConnectionRefused => {
                        error!("ConnectionRefused from Server");
                        return;
                    },
                    ErrorKind::UnexpectedEof => {
                        error!("Connection UnexpectedEoF from Server");
                        return;
                    },
                    _ => {
                        if e.to_string().contains("An existing connection was forcibly closed by the remote host") {
                            error!("Connection forcibly closed by Server");
                            return;
                        }
                        else{
                            error!("Error on receiving data from Server. Error: {}", e);
                            thread::sleep(Duration::from_millis(500));
                            continue;
                        }
                    }
                }
            };
        }
    }

/*----------------------------------------------------------------------------- */

    fn write_to_server(mut stream: TcpStream, rx: mpsc::Receiver<Vec<u8>>){
        loop{

            _ = match rx.recv(){
                Ok(data) => {

                    let _ = match stream.write_all(&data){
                        Ok(_) => (),
                        Err(e) => {
                            error!("Unable to write data to Server. Error: {}", e);
                        }
                    };
                },
                Err(e) => {
                    error!("channel from parser is still closed. Error: {}", e);
                    thread::sleep(Duration::from_millis(2000));
                    continue;
                }
            };
        }
    }
}

/*----------------------------------------------------------------------------- */