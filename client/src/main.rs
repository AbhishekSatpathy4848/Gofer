#![allow(unused)]

use std::char::MAX;
use std::net::{TcpStream};
use std::io::{Read, Write};
// use std::thread::sleep;
use std::{fs, thread, u8};
use std::fs::{File, OpenOptions};
use std::io;
use std::str;

use std::any::type_name;

use aes::cipher::ArrayLength;
use spake2::{Ed25519Group, Identity, Password, Spake2};
use aes::{self, cipher::{generic_array::GenericArray, KeyInit, BlockEncrypt, BlockDecrypt}, Aes128};
use std::net::TcpListener;
#[allow(unused)]

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn send() {
    let send_string = String::from("Send");
    let recv_string = String::from("Recv");
    
    
    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => {
            println!("Successfully connected to server in port 7878");

            let mut send_or_recv:String = String::new();
            println!("Do you want to send or receive? (Send/Recv): ");
            io::stdin().read_line(&mut send_or_recv).unwrap();
            let mut password = String::new();
            
            println!("Enter passphrase");
            io::stdin().read_line(&mut password).unwrap();

            let (s1, message) = Spake2::<Ed25519Group>::start_symmetric(
            &Password::new(password.as_bytes()),
            &Identity::new(b"shared id"));

            if(send_or_recv.trim().eq(&send_string)){
                stream.write_all(password.as_bytes()).unwrap();
                println!("Passphrase sent!!");
                let mut buffer = [0u8; 16];
                stream.read(&mut buffer);
                stream.write_all(&message).unwrap();
                // println!("Outbound message sent");
                println!("Waiting for client 2...");
                //waiting for the inbound message/public key of client 2
                let mut buf = [0u8; 33];
                stream.read(&mut buf);
                println!("Outbound message of client 2 {:?}",buf);
                let encryption_key = s1.finish(&buf).unwrap();
                println!("Encryption key is {:?}",encryption_key);
                // let path = "./src/testing.txt";
                // let mut len = fs::metadata(path).unwrap().len().to_string();
                // let mut msg_len = len.as_bytes();

                // print!("Sent file size: {}\n", len);
                // stream.write_all(&msg_len).unwrap();
                
                send_file(stream,encryption_key);
            }
            else if send_or_recv.trim().eq(&recv_string) {
                stream.write_all(password.as_bytes()).unwrap();
                
                // let mut buffer_passphrase = [0u8; 16]; 
                // stream.read(&mut buffer_passphrase).unwrap();
                // println!("Passphrase{:?}",String::from_utf8_lossy(&buffer_passphrase).to_string());

                // let mut buffer_message = [0u8; 33]; 
                // stream.read(&mut buffer_message).unwrap();
                // let inbound_message = buffer_message.to_vec();
                // println!("Message{:?}",inbound_message);

                
                let mut response = [0u8;4];
                stream.read(&mut response).unwrap();
                // print!("Response from server {}",String::from_utf8_lossy(&response).to_string());
                let str = String::from_utf8_lossy(&response).trim().to_string();
                let str2 = String::from("NACK");
                if(str.eq(&str2)){
                    println!("No matching client found");
                    return;
                }
                
                let mut buffer_message = [0u8; 33]; 
                stream.read(&mut buffer_message).unwrap();
                println!("Outbound message of client 1 is {:?}", buffer_message);
                
                let encryption_key = s1.finish(&buffer_message).unwrap();
                println!("Encryption key is {:?}",encryption_key);

                // println!("Sending {:?}",message);
                stream.write_all(&message).unwrap();

                println!("Waiting for file...");
                // println!("Sent message");
                // stream.read(&mut buffer_message);
                // sleep(std::time::Duration::from_secs(10));

                receive_file(stream,encryption_key);

            }

            // println!("here1");
            // println!("Received{}",String::from_utf8_lossy(&buffer).to_string());

            //let mut msg = message.as_bytes();
            // println!("{}",message);
            // for i in message
            // {
            //     println!("{}",i);
            // }
            // let mut count = 0;
            // let mut buf = [0u8; 8];
            // for i in message {
            //     buf[count] = i;
            //     count+=1;
            // }

            // print!("{:?}",buf);
            
            // let ref_buf = buf.clone();

            // while(buf == ref_buf){}
            // let ref_buf = buf.clone();
            
            // while(buf == ref_buf){}
            
            // ->

            // let mut buf = [0u8; 4];
            // let mut a = (String::from_utf8_lossy(&buf).to_string());

            // while(a.as_str().ne(&send_string)){
            //     stream.read(&mut buf).unwrap();
            //     a = String::from_utf8_lossy(&buf).to_string().trim().to_string();
            //     if(a.as_str().eq(&recv_string)){
            //         break;
            //     }
            // }
            
            // if(a.as_str().eq(&recv_string)){
            //     println!("Waiting for file...");
            //     receive_file(stream);
            //     return;
            // }


            // println!("Would you like to send a text file? (y/n)");
            // let mut input = String::new();
            // io::stdin().read_line(&mut input).unwrap();
            // let yes = String::from("y");

            // if(input.trim().eq(&yes)){
            //     let path = "./src/random.txt";
            //     let mut len = fs::metadata(path).unwrap().len().to_string();
            //     let mut msg_len = len.as_bytes();

            //     print!("Sent file size: {}\n", len);
            //     stream.write_all(&msg_len).unwrap();
                
            //     send_file(stream);

            // }            

            // ->
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
        
    }
    println!("Terminated.");
}

fn send_file(mut stream: TcpStream, key:Vec<u8>) {
    let mut input = String::new();
    
    println!("Enter file name:");
    io::stdin().read_line(&mut input).unwrap();
    // let fullname = "/Users/abhisheksatpathy/gofer/client/src/random.txt";
    let fullname = &input.trim();
    // println!("name: {}",fullname);
    let mut file_size = fs::metadata(fullname).unwrap().len();

    // let mut file_name = "random.txt";
    // let mut fullname = String::from("./src/");
    // fullname.push_str(file_name);
    // println!("FULLPATH: {:?}", fullname);
    let mut len = fs::metadata(fullname).unwrap().len().to_string();
    let mut msg_len = len.as_bytes();
    // let mut remaining_data =  .parse::<i32>().unwrap();
    let modified_size = ((file_size as i32)/16 + 1)*16;
    // print!("Sent file size: {}\n", modified_size);
    stream.write_all(modified_size.to_string().as_bytes()).unwrap();

    //open file in binary mode
    //let mut remaining_data = file_size.parse::<i32>().unwrap();
    let mut remaining_data = file_size as i32;

    let mut buf = [0u8; 16];
    let mut file = File::open(fullname).unwrap();
    let key = GenericArray::from_slice(&key.as_slice()[0..16]);
    let cipher = Aes128::new(&key);
    

    while remaining_data != 0 {
        if remaining_data >= 16
        {
            //read slab from file
            let file_slab = file.read(&mut buf);
            match file_slab{
                Ok(n) => {
                    let mut block = GenericArray::from_mut_slice(&mut buf);
                    // println!("Before text{:?}",block);
                    cipher.encrypt_block(block);
                    // println!("Encrypted text{:?}",block);

                    // println!("abc{:?}",buf);

                    stream.write_all(&block).unwrap();
                    // println!("sent {} file bytes (big)", n);
                    remaining_data = remaining_data - n as i32;
                }
                _ => {}
            }
        }
        else {
            buf = [0u8;16];
            let file_slab = file.read(&mut buf);
            match file_slab {
                //client must shrink this last buffer
                Ok(n) => {
                    
                    let mut block = GenericArray::from_mut_slice(&mut buf);
                    let cipher = Aes128::new(&key);
                    // println!("Before encryption:{:?}",block);
                    // println!("Before text{:?}",block);
                    cipher.encrypt_block(block);
                    // println!("Encrypted text{:?}",block);
                    // println!("Encrypted text{:?}",block);

                    stream.write_all(&block).unwrap();
                    // println!("sent {} file bytes (small)", n);
                    remaining_data = remaining_data - n as i32;
                }
                _ => {}
            }
        }
    }
    println!("Sent file successfully");
}

fn main() {
    send(); 
       
}

fn store_into_file(mut stream: TcpStream){
    // let mut file = File::create("./src/random.txt").unwrap();
    let mut buf = [0u8; 16];
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

fn receive_file(mut stream: TcpStream,key: Vec<u8>) -> String {

    //let mut accumulator: String = String::new();
    let mut r = [0u8; 16]; //16 byte buffer
    
    //read file size
    stream.read(&mut r).unwrap();
    // println!("abhsbaj{:?}",r);
    let msg_len_str = decode_message_size(&mut r);
    // println!("Message length{:?}", msg_len_str);

    let file_name = "recv.txt";
    // let mut fullname = String::from("./src/");
    let mut fullname = "/Users/abhisheksatpathy/gofer/client/src/recv.txt";
    // fullname.push_str(&file_name);

    //create a file

    let mut file_buffer = OpenOptions::new().create(true).append(true).open(fullname).unwrap();
    let key = GenericArray::from_slice(&key.as_slice()[0..16]); 
    let cipher = Aes128::new(&key);

    //receive file itself (write to file)
    let mut remaining_data = msg_len_str.parse::<i32>().unwrap();
    while remaining_data != 0 {
        if remaining_data >= 16 as i32
        {
            let slab = stream.read(&mut r);
            match slab {
                Ok(n) => {
                    let mut block = GenericArray::from_mut_slice(&mut r);
                    // println!("Before text{:?}",block);
                    cipher.decrypt_block(&mut block);
                    // println!("after text{:?}",block);

                    file_buffer.write_all(&mut block).unwrap();
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
                    // let mut r_slice = &r[0..(array_limit as usize + 1)]; //fixes underreading
                    //caused by not using
                    //subprocess call on 
                    //the server
                    let mut block = GenericArray::from_slice(&mut r);
                    let mut block = block.clone();
                    // println!("Before text{:?}",block);
                    cipher.decrypt_block(&mut block);
                    // println!("after text{:?}",block);
                    file_buffer.write_all(&mut block[0..(array_limit as usize + 1)]).unwrap(); //fixes underreading
                   // file_buffer.flush().unwrap();
                    println!("wrote {} bytes to file (small)", remaining_data as i32);
                    remaining_data = 0;
                }
                _ => {}
            }
        }
    }
    println!("Received file successfully");
    String::from("Ok")
}
