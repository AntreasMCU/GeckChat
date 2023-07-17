mod client;

use std::io::{self, BufRead, Write};
use std::net::SocketAddr;
use std::thread;
use log::{trace, info, warn, error}; //debug,error, warn, log_enabled, info, trace, Level};
use env_logger::Env;
use client::TcpClient;
use std::sync::{Arc, Mutex};
use std::ptr;

use std::fs::File;
use std::io::BufReader;
use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink};
use rodio::source::{SineWave, Source};



fn main() {

    let master_name = "Antreas";

    /*----------------------------------------------------- 
            Set up Audio
    ------------------------------------------------------*/

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();


    /*----------------------------------------------------- 
            Set up Logging Enviroment
    ------------------------------------------------------*/
    // The `Env` lets us tweak what the environment
    // variables to read are and what the default
    // value is if they're missing
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    /*----------------------------------------------------- 
            Set up Connection to Server
    ------------------------------------------------------*/

    let address: SocketAddr = "127.0.0.1:3010".parse().expect("Not a valid ServiceManager IP");
    let mut client = TcpClient::new(address).expect("Hard to get an error on tcp_client creation");
    if client.start().is_err(){
        error!("Unable to create TCP managment client");
    }

    let rx = client.get_receiver().expect("Unable to get receiver from client");
    let tx = client.get_sender().expect("Unable to get sender from client");

    //send my name to the server
    let mut connection_msg = vec![0x02, 0x05, 0x83];
    connection_msg.extend_from_slice(master_name.as_bytes());
    tx.send(connection_msg).expect("Unable to send connection_msg");

    let handle = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(s) => {
                    // println!("Received: {}", s);
                    tx.send(s.into_bytes());
                },
                Err(e) => {
                    error!("Error on read line: {}", e);
                }
            }
        }
    });

    loop {
        let data = rx.recv();

        let _ = match data {
            Ok(s) => {
                println!("{}", String::from_utf8_lossy(&s));
                unsafe {
                    // Add a dummy source of the sake of the example.
                    let sound_source = SineWave::new(440).take_duration(Duration::from_secs_f32(0.02)).amplify(0.20);
                    sink.append(sound_source);
                    let sound_source = SineWave::new(400).take_duration(Duration::from_secs_f32(0.04)).amplify(0.20);
                    sink.append(sound_source);
                    let sound_source = SineWave::new(440).take_duration(Duration::from_secs_f32(0.02)).amplify(0.20);
                    sink.append(sound_source);
                }
            },
            Err(e) => {
                error!("Error on receiving from tcp : {}", e);
            }
        };
    }

}
