//single threaded server for now
use std::net::{TcpListener, TcpStream};
// use log;
use std::{thread, fs};
use std::fs::File;
use std::io::{Write, Read};
mod handler;

fn main() {
    let path = "./src/recv.txt";
    fs::write(path, "");
    let addr = "127.0.0.1:7878";
    let server = TcpListener::bind(&addr).unwrap();
    let mut client_count = 0;

    // log::info!("Listening on {addr}");

    let (mut tx,rx) = spmc::channel();

    for stream in server.incoming() {
        match stream {
            Ok(mut stream) => {
                client_count+=1;
                if client_count != 1 {tx.send(client_count).unwrap()};
                println!("New connection: {}", stream.peer_addr().unwrap());

                let rx = rx.clone();
                
                thread::spawn(move|| {
                    // connection succeeded
                    if client_count == 2 {
                        stream.write_all(String::from("Recv").as_bytes()).unwrap();
                        send_file(stream);
                        return;
                    }
                    while rx.recv().unwrap() != 2 {};
                    stream.write_all(String::from("Send").as_bytes()).unwrap();
                    handler::handle_incoming_conn(stream);
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


fn send_file(mut stream: TcpStream) {
    let path = "./src/recv.txt";
    let mut file_size = fs::metadata(path).unwrap().len();
    while file_size == 0 {
        file_size = fs::metadata(path).unwrap().len();
    }
    print!("File size : {}\n", file_size);

    let file_name = "recv.txt";
    let mut fullname = String::from("./src/");
    fullname.push_str(file_name);
    println!("FULLPATH: {:?}", fullname);

    let mut remaining_data = file_size as i32;
    stream.write_all(remaining_data.to_string().as_bytes()).unwrap();

    let mut buf = [0u8; 8];
    let mut file = File::open(fullname).unwrap();

    while remaining_data != 0 {
        if remaining_data >= 8
        {
            //read slab from file
            let file_slab = file.read(&mut buf);
            match file_slab{
                Ok(n) => {
                    stream.write_all(&buf).unwrap();
                    println!("sent {} file bytes (big)", n);
                    remaining_data = remaining_data - n as i32;
                }
                _ => {}
            }
        }
        else {
            let file_slab = file.read(&mut buf);
            match file_slab {
                Ok(n) => {
                    stream.write_all(&buf).unwrap();
                    println!("sent {} file bytes (small)", n);
                    remaining_data = remaining_data - n as i32;
                }
                _ => {}
            }
        }
    }
}