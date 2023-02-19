#![allow(unused)]

use std::net::{TcpStream};
use std::io::{Read, Write};
use std::{fs, thread};
use std::fs::{File, OpenOptions};
use std::io;
use std::str;

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

            let mut buf = [0u8; 4];
            // let ref_buf = buf.clone();

            // while(buf == ref_buf){}

            let mut a = (String::from_utf8_lossy(&buf).to_string());

            let mut send_string = String::from("Send");
            let mut recv_string = String::from("Recv");
            while(a.as_str().ne(&send_string)){
                stream.read(&mut buf).unwrap();
                a = String::from_utf8_lossy(&buf).to_string().trim().to_string();
                if(a.as_str().eq(&recv_string)){
                    break;
                }
            }
            
            if(a.as_str().eq(&recv_string)){
                println!("Waiting for file...");
                receive_file(stream);
                return;
            }


            println!("Would you like to send a text file? (y/n)");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let yes = String::from("y");

            if(input.trim().eq(&yes)){
                let path = "./src/random.txt";
                let mut len = fs::metadata(path).unwrap().len().to_string();
                let mut msg_len = len.as_bytes();

                print!("Sent file size: {}\n", len);
                stream.write_all(&msg_len).unwrap();
                
                send_file(stream);

            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}


fn store_into_file(mut stream: TcpStream){
    // let mut file = File::create("./src/random.txt").unwrap();
    let mut buf = [0u8; 8];
    stream.read(&mut buf).unwrap();
    let recv = String::from_utf8_lossy(&buf);
    println!("Received from the server: {}", recv);

}

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

    //let mut accumulator: String = String::new();
    let mut r = [0u8; 8]; //8 byte buffer
    
    //read file size
    stream.read(&mut r).unwrap();
    let msg_len_str = decode_message_size(&mut r);
    println!("Message length{:?}", msg_len_str);

    let file_name = "recv.txt";
    let mut fullname = String::from("./src/");
    fullname.push_str(&file_name);

    //create a file

    let mut file_buffer = OpenOptions::new().create(true).append(true).open(fullname).unwrap();

    //receive file itself (write to file)
    let mut remaining_data = msg_len_str.parse::<i32>().unwrap();
    while remaining_data != 0 {
        if remaining_data >= 8 as i32
        {
            let slab = stream.read(&mut r);
            match slab {
                Ok(n) => {
                    file_buffer.write_all(&mut r).unwrap();
                    //file_buffer.flush().unwrap();
                    println!("wrote {} bytes to file", n);
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
                    println!("wrote {} bytes to file (small)", remaining_data as i32);
                    remaining_data = 0;
                }
                _ => {}
            }
        }
    }
    String::from("Ok")
}