#![allow(unused)]

use std::net::{TcpStream};
use std::io::{Read, Write};
use std::fs;
use std::fs::File;
use encoding::{Encoding, EncoderTrap};
use encoding::all::ASCII;
use std::any::type_name;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn send_file(mut stream: TcpStream) {
    let path = "./src/random.txt";
    let mut file_size = fs::metadata(path).unwrap().len();

    let mut file_name = "random.txt";
    let mut fullname = String::from("./src/");
    fullname.push_str(file_name);
    println!("FULLPATH: {:?}", fullname);

    //open file in binary mode
    //let mut remaining_data = file_size.parse::<i32>().unwrap();
    let mut remaining_data = file_size as i32;

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
                //client must shrink this last buffer
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

fn main() {

    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 7878");

            let path = "./src/random.txt";
            let mut len = fs::metadata(path).unwrap().len().to_string();
            let mut msg_len = len.as_bytes();

            stream.write_all(&msg_len).unwrap();
            
            send_file(stream);
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
