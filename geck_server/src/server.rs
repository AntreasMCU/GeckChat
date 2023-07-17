use std::{thread, time, io};
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::error::Error;
use std::io::{Read, Write, ErrorKind};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use log::{debug, error, warn, info};
// use tokio::sync::oneshot::error::RecvError; //log_enabled, info, trace, Level};

#[derive(Debug)]
pub struct ConnectionInfo {
    info: HashMap<SocketAddr, TcpStream>
}

impl ConnectionInfo {
    pub fn new() -> ConnectionInfo{
        ConnectionInfo {
            info: HashMap::new()
        }
    }
}

/*-------------------------------------------------------------
---------------------------------------------------------------
--------------------------------------------------------------- */

pub struct TcpServer {
    addr: SocketAddr,
    _port_active: Arc<AtomicBool>,
    connections: Arc<Mutex<ConnectionInfo>>,
    _connections_handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    _to_parser_addr_rx: Option<mpsc::Receiver<(SocketAddr, Vec<u8>)>>,
    to_parser_addr_tx: Option<mpsc::Sender<(SocketAddr, Vec<u8>)>>,
    _to_parser_rx: Option<mpsc::Receiver<Vec<u8>>>,
    to_parser_tx: Option<mpsc::Sender<Vec<u8>>>,
    from_parser_addr_rx: Option<mpsc::Receiver<(SocketAddr, Vec<u8>)>>,
    _from_parser_addr_tx: Option<mpsc::Sender<(SocketAddr, Vec<u8>)>>,
    from_parser_rx: Option<mpsc::Receiver<Vec<u8>>>,
    _from_parser_tx: Option<mpsc::Sender<Vec<u8>>>,
    addr_based_server: bool
}

/*-------------------------------------------------------------
---------------------------------------------------------------
--------------------------------------------------------------- */

impl TcpServer {
    pub fn new(addr: SocketAddr, addressing_enabled: bool) -> Result<TcpServer, Box<dyn Error>> {
        // Perform any necessary checks on the `addr` argument
        let (to_parser_addr_tx , to_parser_addr_rx) = mpsc::channel::<(SocketAddr, Vec<u8>)>();
        let (to_parser_tx , to_parser_rx) = mpsc::channel::<Vec<u8>>();
        let (from_parser_addr_tx , from_parser_addr_rx) = mpsc::channel::<(SocketAddr, Vec<u8>)>();
        let (from_parser_tx , from_parser_rx) = mpsc::channel::<Vec<u8>>();
        // let (socket_health_tx , socket_health_rx) = mpsc::channel::<SocketAddr>();
        Ok(TcpServer {
            addr,
            _port_active: Arc::new(AtomicBool::new(true)),
            connections: Arc::new(Mutex::new(ConnectionInfo::new())),
            _connections_handles: Arc::new(Mutex::new(Vec::new())),
            _to_parser_addr_rx: Some(to_parser_addr_rx),
            to_parser_addr_tx: Some(to_parser_addr_tx),
            _to_parser_rx: Some(to_parser_rx),
            to_parser_tx: Some(to_parser_tx),
            from_parser_addr_rx: Some(from_parser_addr_rx),
            _from_parser_addr_tx: Some(from_parser_addr_tx),
            from_parser_rx: Some(from_parser_rx),
            _from_parser_tx: Some(from_parser_tx),
            addr_based_server: addressing_enabled
        })
    }

/*----------------------------------------------------------- */

    pub fn _get_rx_receiver(&mut self) -> mpsc::Receiver<Vec<u8>>{
        // let shared_parser_rx = self.parser_rx.lock().await;
        // return shared_parser_rx;
        self._to_parser_rx.take().expect("No available Receiver to be returned")
    }

/*----------------------------------------------------------- */

    pub fn _get_tx_sender(&mut self) -> mpsc::Sender<Vec<u8>>{
        // let shared_parser_tx = &self.parser_tx;
        self._from_parser_tx.take().expect("No available Sender to be returned")
    }

/*----------------------------------------------------------- */

    pub fn get_addr_rx_receiver(&mut self) -> mpsc::Receiver<(SocketAddr, Vec<u8>)>{
        // let shared_parser_rx = self.parser_rx.lock().await;
        // return shared_parser_rx;
        self._to_parser_addr_rx.take().expect("No available Receiver to be returned")
    }

/*----------------------------------------------------------- */

    pub fn get_addr_tx_sender(&mut self) -> mpsc::Sender<(SocketAddr, Vec<u8>)>{
        // let shared_parser_tx = &self.parser_tx;
        self._from_parser_addr_tx.take().expect("No available Sender to be returned")
    }

    /*------------------------------------------------------------------------------------------
             +-------------------------+     mpsc: to_parser<(socketAddr, Vec<u8>)>    +-----------------------+ 
      -->    |                         |->|  or mpsc: to_parser<Vec<u8>>               |                       |
      -->    |        Sockets recv     +->|  ---------------------------->---------->  +        Parser         |
      -->    |                         |->|                               |            |                       |
             +-------------------------+                                  |            +-----------------------+
                           | | |                                          |              |
                           -----     mpsc: close_alert                    |            mpsc: to_router<(socketAddr, Vec<u8>)>
                             |--------------------|                       |                         or
                                                  |                       |            mpsc: to_router<Vec<u8>>
                                                  V                       |              |  
                                              +-------------------+       |              |
            <---|  vec<mpsc>:mpsc: to_sockets |                   |       |              |
            <---|---------------------------- |  Sockets Transmit |<-------<-------------|
            <---|                             |                   |       |
                                              +-------------------+       |
                                                                          |
                                                                          |
    --------------------------------------------------------------------------------------------*/

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        
        debug!("Entering wait connection");
        

        //create a new struct holding the connections
        let connections = Arc::new(Mutex::new(ConnectionInfo::new()));
        // let shared_connections = Arc::clone(&connections);

        let shared_to_parser_tx = self.to_parser_tx.take().expect("No available to_parser_tx");
        let shared_to_parser_addr_tx = self.to_parser_addr_tx.take().expect("No available to_parser_addr_tx");

        // let shared_connections_handles = Arc::clone(&self.connections_handles);
        
        let shared_addr_based_server = self.addr_based_server;
        // let shared_port_active = Arc::clone(&self.port_active);
        let listen_connections = Arc::clone(&connections);
        let shared_addr = self.addr;
        info!("Shared_address: {:?}", shared_addr);
        
        thread::sleep(time::Duration::from_millis(100));
        thread::spawn(move || {           

            //while shared_port_active.load(Ordering::Relaxed) {
                Self::listen_for_connection(
                    // shared_listener,
                    shared_addr,
                    listen_connections, 
                    shared_addr_based_server,
                    shared_to_parser_tx,
                    shared_to_parser_addr_tx
                );
        });

        let shared_from_parser_rx = self.from_parser_rx.take().expect("No available from_parser.rx");
        let shared_from_parser_addr_rx = self.from_parser_addr_rx.take().expect("No available from_parser_addr_rx");
        let router_addr_based_server = self.addr_based_server;
        let shared_connections = Arc::clone(&connections);

        let _route_handle = thread::spawn(move || {
            Self::handle_outbound_packets(
                shared_from_parser_rx,
                shared_from_parser_addr_rx,
                shared_connections,
                router_addr_based_server
            );
        });

        info!("Exiting wait connection");

        Ok(())
    }

    /*--------------------------------------------------------------------------------------------*/

    fn listen_for_connection(
        connection_address: SocketAddr, 
        connections: Arc<Mutex<ConnectionInfo>>, 
        addr_based_server: bool,
        to_parser_tx: mpsc::Sender<Vec<u8>>,
        to_parser_addr_tx: mpsc::Sender<(SocketAddr, Vec<u8>)>

    ){
        let mut connections_handles = Vec::new();

        let listener = TcpListener::bind(&connection_address).expect("unable to bind to port");        
        // let shared_listener = Arc::new(Mutex::new(listener));

        // let listener_lock = listener.lock().expect("Error on locking Listener");

        loop{
            // info!("Waiting for Connection");
            let (socket, address) = match listener.accept() {
                Ok(result) => result,
                Err(err) => {
                    error!("Error on accept connection {}", err);
                    continue;
                }
            };
            info!("Connected to port. Port Info: {:?}", address);
            let thread_addr_based_server = addr_based_server;

            let thread_to_parser_tx = mpsc::Sender::clone(&to_parser_tx);
            let thread_to_parser_addr_tx = mpsc::Sender::clone(&to_parser_addr_tx);
            
            // clone all Arcs to pass to the socket task
            // let port_active_to_socket = port_active_listener.clone();
            // let task_shared_to_parser_tx = shared_to_parser_tx.clone();   
            // let task_sender_close_alert = shared_sender_close_alert.clone();

            // let shared_to_socket_tx = Arc::new(to_socket_tx);

            let shared_socket = socket.try_clone().expect("Unable to unwrap socket");
            let thread_connections = Arc::clone(&connections);

            let handle = thread::spawn(move || {
                Self::handle_incoming_packet(
                    shared_socket, 
                    thread_to_parser_tx.clone(),
                    thread_to_parser_addr_tx.clone(),
                    thread_connections,
                    thread_addr_based_server
                );
            });

            let mut connections_lock = connections.lock().expect("Unable to unlock connections to append new connection");
            connections_lock.info.insert(address, socket);
            // debug!("New Hashmap: {:?}", connections_lock.info);

            connections_handles.push(handle);
        }
    }

/*--------------------------------------------------------------------------------------------*/

    fn handle_outbound_packets(mut _rx: mpsc::Receiver<Vec<u8>>, addr_rx: mpsc::Receiver<(SocketAddr, Vec<u8>)>, connections: Arc<Mutex<ConnectionInfo>>, addr_based_server: bool) -> JoinHandle<()> { //, rx: mpsc::Receiver<(SocketAddr, Vec<u8>)>
     
        info!("Starting internal_packet_router thread");

        loop{

            if addr_based_server {
                _ = match addr_rx.recv(){
                    Ok(data) => {
                        let (addr, data) = data;

                        let connections = match connections.lock() {
                            Ok(d) => d,
                            Err(e) => {
                                error!("Unable to unlock connections hashmap for retriving tcpStream. Error: {:?}", e);
                                continue;
                            }
                        };

                        let mut socket = match connections.info.get(&addr) {
                            Some(socket) => socket,
                            None => {
                                error!("Unable to retrieve stream for addr: {:?}", addr);
                                error!("HashMap: {:?}", connections.info);
                                continue;
                            }
                        };

                        let _ = match socket.write_all(&data){
                            Ok(_) => (),
                            Err(e) => {
                                error!("Unable to write data to socket {}. Error: {}", addr, e);
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
            else{
                _ = match _rx.recv() {
                    Ok(data) => {
                        let connections = connections.lock().expect("unable to unlock connections for tx itterating");

                        for (_addr, mut stream) in connections.info.iter(){
                            stream.write_all(&data).expect("Failed to send data to addr:");                                
                        }
                    },
                    Err(e) => {
                        error!("Error on retriving data from parser. Error: {}", e);
                        continue;
                    }
                };
            }
        }

        // info!("Exit router thread");       
    }

/*--------------------------------------------------------------------------------------------*/

    fn handle_incoming_packet(
        mut socket: TcpStream,
        tx_to_parser: mpsc::Sender<Vec<u8>>,
        tx_addr_to_parser: mpsc::Sender<(SocketAddr, Vec<u8>)>,
        connections: Arc<Mutex<ConnectionInfo>>,
        addr_based_server: bool
    )
    {
    
        //get local address from the socket
        let local_addr = match socket.peer_addr(){
            Ok(addr) => addr,
            Err(e) => {
                error!("Error exporting local address from socket: {}", e);
                return;
            }
        };
        
        info!("Handle Connection {:?} is up", socket);
    
        loop {
            let mut buf: Vec<u8> = vec![0; 1024];

            //read data from TCP socket and send them to parser stream
            if addr_based_server {
                let _ = match socket.read(&mut buf){
                    Ok(n) => {
                        if n == 0 {
                            // connection closed
                            warn!("Connection closed from Client {}", local_addr);
                            if connections.lock().expect("unable to lock connections to erase connection").info.remove(&local_addr).is_none(){
                                error!("Unable to send close connection to router");
                            }
                            return;
                        }
                        else{
                            //reduce buffer size to minimum length. Data are copied so we need to be smaller in size
                            //truncate takes care of n==0 or n > buf.length
                            buf.truncate(n);
                            // info!("packet arrived. Data: {:?}", buf);
                            tx_addr_to_parser.send((local_addr, buf)).expect("Erro on sending addr_packet to parser");
                        }
                    },
                    Err(e) => match e.kind() {
                        io::ErrorKind::ConnectionAborted => {
                            warn!("ConnectionAborted from Client {}", local_addr);
                            if connections.lock().expect("unable to lock connections to erase connection").info.remove(&local_addr).is_none(){
                                tx_addr_to_parser.send((local_addr, vec![])).expect("Erro on sending addr_packet to parser");
                                error!("Unable to remove connection from connections Hashmap");
                            }
                            return;
                        },
                        io::ErrorKind::ConnectionRefused => {
                            warn!("ConnectionRefused from Client {}", local_addr);
                            if connections.lock().expect("unable to lock connections to erase connection").info.remove(&local_addr).is_none(){
                                tx_addr_to_parser.send((local_addr, vec![])).expect("Erro on sending addr_packet to parser");
                                error!("Unable to remove connection from connections Hashmap");
                            }
                            return;
                        },
                        io::ErrorKind::UnexpectedEof => {
                            warn!("Connection UnexpectedEoF from Client {}", local_addr);
                            if connections.lock().expect("unable to lock connections to erase connection").info.remove(&local_addr).is_none(){
                                tx_addr_to_parser.send((local_addr, vec![])).expect("Erro on sending addr_packet to parser");
                                error!("Unable to remove connection from connections Hashmap");
                            }
                            return;
                        },
                        _ => {
                            if e.to_string().contains("An existing connection was forcibly closed by the remote host") {
                                warn!("Connection forcibly closed by Client {}", local_addr);
                                tx_addr_to_parser.send((local_addr, vec![])).expect("Erro on sending addr_packet to parser");
                                if connections.lock().expect("unable to lock connections to erase connection").info.remove(&local_addr).is_none(){
                                    error!("Unable to remove connection from connections Hashmap");
                                }
                                return;
                            }
                            else{
                                error!("Error on receiving data from addr: {:?}. Error: {}", local_addr, e);
                                thread::sleep(Duration::from_millis(500));
                                continue;
                            }
                        }
                    }
                };
            }
            else{
                let _ = match socket.read(&mut buf){
                    Ok(0) => {
                        // connection closed
                       warn!("Connection closed from Client {}", local_addr);
                       if connections.lock().expect("unable to lock connections to erase connection").info.remove(&local_addr).is_none(){
                           error!("Unable to send close connection to router");
                       }
                       return;
                   }
                    Ok(_n) => {
                        tx_to_parser.send(buf).expect("Erro on sending packet to parser");
                    },
                    Err(e) => {
                        error!("Error on receiving data from addr: {:?}. Error: {}", local_addr, e);
                        continue;
                    }
                };
            }
        }
    }
}

/*-------------------------------------------------------------
---------------------------------------------------------------
--------------------------------------------------------------- */