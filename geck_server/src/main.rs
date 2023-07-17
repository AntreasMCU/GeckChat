// mod json_reader;
mod server;
mod dispatcher;

use log::{trace, info, warn, error}; //debug,error, warn, log_enabled, info, trace, Level};
use dispatcher::Dispatcher;
use server::TcpServer;
use env_logger::Env;

fn main(){


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
            Set up TCP server
------------------------------------------------------*/

let address = "127.0.0.1:3010"
                            .parse()
                            .expect("Not a valid IP");
    
let mut server = TcpServer::new(address, true).expect("TCP server failed");

match server.start(){
    Ok(_) => {},
    Err(err) => {
        println!("Ti fasi? Error edo? {}", err);
    }
};

/*----------------------------------------------------- 
            Set up TCP Dispatcher
------------------------------------------------------*/

let mut dispatcher = Dispatcher::new(server.get_addr_rx_receiver(), server.get_addr_tx_sender()).expect("Failed to pass arguments to server parser");
dispatcher.run().expect("We got error from server_parser.run :/");//("server parser returned error");

}