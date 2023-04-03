#![allow(unused)]

use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str;
use std::io::BufWriter;
use std::fs::File;
use std::fs::OpenOptions;

fn decode_message_size(mut ack_buf: &mut [u8]) -> String {
    let msg_len_slice: &str = str::from_utf8(&mut ack_buf).unwrap();
    let mut msg_len_str = msg_len_slice.to_string();
    let mut numeric_chars = 0;
    for c in msg_len_str.chars() {
        if c.is_numeric() == true {
            numeric_chars = numeric_chars + 1;
        }
    }
    //shrink:
    msg_len_str.truncate(numeric_chars);
    msg_len_str
}

fn receive_file(mut stream: TcpStream) -> String {

    // println!("Inside Recv function");

    //let mut accumulator: String = String::new();
    let mut r = [0u8; 16]; //8 byte buffer
    
    //read file size
    stream.read(&mut r).unwrap();
    let msg_len_str = decode_message_size(&mut r);
    // println!("{:?}", msg_len_str);

    let file_name = "/Users/abhisheksatpathy/gofer/server/src/recv.txt";
    // let mut fullname = String::from("./src/");
    // fullname.push_str(&file_name);

    //create a file

    let mut file_buffer = OpenOptions::new().create(true).append(true).open(file_name).unwrap();

    //receive file itself (write to file)
    let mut remaining_data = msg_len_str.parse::<i32>().unwrap();
    while remaining_data != 0 {
        if remaining_data >= 16 as i32
        {
            let slab = stream.read(&mut r);
            match slab {
                Ok(n) => {
                    file_buffer.write_all(&mut r).unwrap();
                    //file_buffer.flush().unwrap();
                    // println!("wrote {} bytes to file", n);
                    remaining_data = remaining_data - n as i32;
                }
                _ => {}
            }
        } else {
            let array_limit = (remaining_data as i32) - 1;
            let slab = stream.read(&mut r);
            match slab {
                Ok(_) => {
                    let mut r_slice = &r[0..(array_limit as usize + 1)]; //fixes underreading
                    //caused by not using
                    //subprocess call on 
                    //the server
                    file_buffer.write_all(&mut r_slice).unwrap();
                   // file_buffer.flush().unwrap();
                    // println!("wrote {} bytes to file (small)", remaining_data as i32);
                    remaining_data = 0;
                }
                _ => {}
            }
        }
    }
    String::from("Ok")
}

pub fn handle_incoming_conn(mut stream: TcpStream) {
    
    // let mut data = [0 as u8; 16]; // using 8 byte buffer

    // stream.read(&mut data).unwrap();
    // let msg_len_str = decode_message_size(&mut data);

    // println!("Message size is {}", msg_len_str);
    // println!("Inside handle Incoming Connections(Recv)");
    receive_file(stream);
}

fn main(){}