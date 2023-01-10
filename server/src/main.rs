//single threaded server for now
use std::net::TcpListener;
use log;
use std::thread;

mod handler;

fn main() {
    let addr = "127.0.0.1:7878";
    let server = TcpListener::bind(&addr).unwrap();

    log::info!("Listening on {addr}");

    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handler::handle_incoming_conn(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }

    }
    
    drop(server);
}
