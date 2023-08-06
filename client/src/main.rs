use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::{fs, u8, thread};
use std::fs::{File, OpenOptions};
use std::io;
use std::str;


use sha2::Sha256;
use indicatif::{ProgressBar, ProgressStyle};
use hkdf::Hkdf;
use spake2::{Ed25519Group, Identity, Password, Spake2};
use aes::{self, cipher::{generic_array::GenericArray, KeyInit, BlockEncrypt, BlockDecrypt}, Aes128};
use block_padding::{Pkcs7, Padding};

const SEND: &str = "send";
const RECV: &str = "recv";
const ACK: &str = "ack "; //ensure this is 4 bytes long
const NACK: &str = "nack";
const DONE: &str = "done";
const SERVER_IP_ADDRESS: &str = "127.0.0.1:8080";
const OUTPUT_FILE_PATH: &str = "/app/client/output/recv_file.pdf";


fn trim_buffer(buffer: &[u8]) -> &[u8] {
    let last_non_zero_index: usize = buffer.iter().rposition(|&x| x != 0).map(|index| index + 1).unwrap();
    &buffer[0..last_non_zero_index]
}

fn convert_to_128_bit_key(key: &[u8], passphrase: String) -> [u8;16] {
    // let mut hasher = Sha256::new();
    // hasher.update(&key);
    // let result = hasher.finalize();
    // let mut compressed_key = [0u8;16];
    // compressed_key.copy_from_slice(&result[..16]);
    // compressed_key
    let mut compressed_key = [0u8; 16];

    let hk = Hkdf::<Sha256>::new(Some(passphrase.as_bytes()), key);

    hk.expand(&passphrase.as_bytes(), &mut compressed_key).unwrap();

    compressed_key
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
    msg_len_str.truncate(numeric_chars);
    msg_len_str
}

fn padd_block(data: &[u8;16], data_size: usize) -> [u8;16]{
    // println!("Data size {}",data_size);
    if data_size == 16 {
        return *data;
    }
    let mut block = [0u8; 16];
    // println!("Data size {}",data_size);
    for i in 0..data_size {
        block[i] = data[i];
    }
    Pkcs7::pad(&mut block, data_size, 16).unwrap();
    block
}

fn unpadd_block(data: &[u8]) -> &[u8]{
    Pkcs7::unpad(data).unwrap()
}

fn main() {
    match TcpStream::connect(SERVER_IP_ADDRESS) {
        Ok(mut stream) => {
            println!("Successfully connected to server on port 8080");

            let mut send_or_recv:String = String::new();
            println!("Do you want to send or receive? (Send/Recv): ");
            io::stdin().read_line(&mut send_or_recv).unwrap();
            send_or_recv = String::from(&send_or_recv[0..4]);
            stream.write_all(send_or_recv.as_bytes()).unwrap();

            
            if(SEND.eq(&send_or_recv.to_ascii_lowercase())){
                
                let mut buffer = [0u8; 16];
                stream.read(&mut buffer).unwrap();
                let passphrase:String = String::from_utf8_lossy(&buffer).to_string();
                let passphrase = passphrase.trim_matches(char::from(0)).to_string();
                println!("Enter this passphrase on another client -> {}", passphrase);

                println!("Waiting for client 2...");

                let mut buffer = [0u8; 16];
                stream.read(&mut buffer).unwrap();
                let ack_string = String::from_utf8_lossy(&buffer).to_string();
                let ack_string = ack_string.trim_matches(char::from(0)).to_string();

                if(ACK.eq(&ack_string)){
                    //key exchange begins
                    
                    //generate keys
                    println!("PASSPHRASE {:?}", passphrase.as_bytes());
                    let (s1, message) = Spake2::<Ed25519Group>::start_symmetric(
                        &Password::new(passphrase.as_bytes()),
                        &Identity::new(b"shared id"));
                        
                    //send public key of client to server
                    stream.write_all(&message).unwrap();

                    println!();
                    
                    //waiting for public key of client 2
                    let mut public_key_client_2_buffer = [0u8;64];
                    stream.read(&mut public_key_client_2_buffer).unwrap();
                    
                    let mut public_key_client_2_buffer = trim_buffer(&public_key_client_2_buffer);
                    println!("Public key of client 2 {:?}",public_key_client_2_buffer);
                    
                    //generate shared symmetric key
                    let encryption_key = s1.finish(&public_key_client_2_buffer).unwrap();
                    println!();
                    println!("Encryption key is {:?}",encryption_key);
                    println!();

                    //key exchange complete

                    //convert the key from 33 bytes to 16 bytes for AES-128
                    let encryption_key = convert_to_128_bit_key(&encryption_key, passphrase);
                
                    println!("Key exchange complete. Ready to send file...");

                    println!();

                    //read to send file
                    send_file(stream,encryption_key);
                }else{
                    println!("Client 2 did not connect");
                    return;
                }
            }
            else if (RECV.eq(&send_or_recv.to_ascii_lowercase())){ {
                let mut passphrase_recv:String = String::new();
                // stream.read(&mut buffer).unwrap();
                println!("Enter the passphrase to receive the file");
                io::stdin().read_line(&mut passphrase_recv).unwrap();
                for i in 0..passphrase_recv.len(){
                    if(passphrase_recv.as_bytes()[i] == 10){
                        passphrase_recv.remove(i); //i specifies the byte index 
                    }
                }
                passphrase_recv = String::from(passphrase_recv.trim_matches(char::from(0)));
                stream.write_all(passphrase_recv.as_bytes()).unwrap();
                
                let mut buffer = [0u8; 16];
                stream.read(&mut buffer).unwrap();

                let ack_string = String::from_utf8_lossy(&buffer).to_string();
                let ack_string = ack_string.trim_matches(char::from(0)).to_string();

                println!("buffer {:?}", buffer);

                if(NACK.eq(&ack_string)){
                    println!("No matching passphrase found");
                    return;
                }
                println!("ACK received");

                //key exchange begins

                //generate keys
                println!("PASSPHRASE {:?}", passphrase_recv.as_bytes());
                let (s1, message) = Spake2::<Ed25519Group>::start_symmetric(
                    &Password::new(passphrase_recv.as_bytes()),
                    &Identity::new(b"shared id"));
                

                //waiting for public key of client 1
                let mut public_key_client_1_buffer = [0u8;64];
                stream.read(&mut public_key_client_1_buffer).unwrap();
                
                let mut public_key_client_1_buffer = trim_buffer(&public_key_client_1_buffer);
                println!("Public key of client 1 {:?}",public_key_client_1_buffer);
                println!();
                    

                //send public key of client to server
                stream.write_all(&message).unwrap();

                //generate shared symmetric key
                let encryption_key = s1.finish(&public_key_client_1_buffer).unwrap();
                println!("Encryption key is {:?}",encryption_key);
                println!();

                //key exchange complete

                //convert the key from 33 bytes to 16 bytes for AES-128
                let encryption_key = convert_to_128_bit_key(&encryption_key, passphrase_recv);

                println!("Key exchange complete. Ready to receive file...");

                receive_file(stream, encryption_key);
                
            }
        }
        else{
            println!("Invalid input");
            return;
        }
    }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
        
    }
    println!("Terminated.");
}

fn send_file(mut stream: TcpStream, key:[u8;16]) {
    let mut input_file_name = String::new();
    
    println!("Enter file name:");
    io::stdin().read_line(&mut input_file_name).unwrap();

    let input_file_name = input_file_name.trim();

    let input_file_size = fs::metadata(input_file_name).unwrap().len();
    println!("File size is {:.2}KB",(input_file_size as f32)/1024.0);

    //we are sending data only in blocks of 16 bytes
    // let mut modified_file_size = (input_file_size / 16) as u32 * 16;
    // if(input_file_size % 16 != 0){
    //     modified_file_size += 16;
    // }

    //send the modified file size to server
    stream.write_all(input_file_size.to_string().as_bytes()).unwrap();

    let mut remaining_data = input_file_size as i32;

    let mut file = File::open(input_file_name).unwrap();

    let cipher = Aes128::new(&GenericArray::from_slice(&key));

    let bar = ProgressBar::new(remaining_data as u64);

    let progress_style_bar = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes}")
        .progress_chars("=>-");

    bar.set_style(progress_style_bar);


    while remaining_data != 0 {
        // println!("Remaining data {}",remaining_data);
        
        if remaining_data >= 16
        {
            let mut buf = [0u8;16];
            let file_slab = file.read_exact(&mut buf);
            match file_slab{
                Ok(n) => {
                    
                    let block = GenericArray::from_mut_slice(&mut buf);
                    
                    cipher.encrypt_block(block);
                    
                    //send the encrypted block to server
                    stream.write_all(&block).unwrap();
                    
                    remaining_data = remaining_data - 16;
                    
                    bar.inc(16 as u64);
                    
                }
                _ => {}
            }
        }
        else {
            let mut buf = [0u8;16];
            let file_slab = file.read(&mut buf);
            match file_slab {
                Ok(n) => {
                    
                    let mut padded_block = padd_block(&buf, n as usize);
                    
                    let block = GenericArray::from_mut_slice(&mut padded_block);
                    
                    cipher.encrypt_block(block);
                    
                    //send the encrypted block with padding to server 
                    stream.write_all(&block).unwrap();
                    
                    remaining_data = remaining_data - n as i32;

                    bar.inc(n as u64);
                }
                e => {
                    println!("Error reading file {:?}", e);
                }
            }
        }
        //wait for ack fromt the server that the receiver has received the block
        let mut ack_buffer = [0u8;4];
        stream.read(&mut ack_buffer).unwrap();
        let ack_string = String::from_utf8_lossy(&ack_buffer).to_string().trim_matches(char::from(0)).to_string();
        if ACK.ne(&ack_string) {
            // println!("Received ACK from server to send the next block");
            println!("Received NACK from server");
            return;
        }
    }
    bar.finish();
    stream.write_all(DONE.as_bytes()).unwrap();
    
    println!("Sent file successfully");
    thread::sleep(Duration::from_secs(1));
}


fn receive_file(mut stream: TcpStream,key: [u8;16]) {
    
    let mut file_length = [0u8; 16];
    
    stream.read(&mut file_length).unwrap();
    
    stream.write_all(ACK.as_bytes()).unwrap();
    
    let msg_len_str = decode_message_size(&mut file_length);

    let output_file_name = OUTPUT_FILE_PATH;
    
    let mut file = OpenOptions::new().create(true).append(true).open(output_file_name).unwrap();
    
    let cipher = Aes128::new(&GenericArray::from_slice(&key.as_slice()));
    
    let mut remaining_data = msg_len_str.parse::<i32>().unwrap();
    
    let bar = ProgressBar::new(remaining_data as u64);
    
    let progress_style_bar = ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes}")
    .progress_chars("=>-");

    bar.set_style(progress_style_bar);


while remaining_data > 0 {
    let mut buf = [0u8;16];
    let data: Result<usize, io::Error> = stream.read(&mut buf);
    
    match data{
        //n will always be 16
        Ok(n) => {
                // println!("remaining data {}",remaining_data);

                if remaining_data >= 16 {
                  
                    let mut block = GenericArray::from_mut_slice(&mut buf);
                    
                    cipher.decrypt_block(&mut block);
                    
                    file.write(&mut block).unwrap();

                    bar.inc(16 as u64);
                    
                }
                else{
                    
                    let mut block = GenericArray::from_mut_slice(&mut buf);
                    
                    cipher.decrypt_block(&mut block);
                    
                    let mut unpadded_block = unpadd_block(&block);
                    
                    file.write(&mut unpadded_block).unwrap();

                    bar.inc(remaining_data as u64);
                    
                }
                
                remaining_data = remaining_data - 16;
                stream.write_all(ACK.as_bytes()).unwrap_or_else(|_| {
                    println!("Error in sending ACK");
                    return;
                });
            }
            _ => {}
        }
    }
    let mut done_buffer = [0u8;4];
    stream.read(&mut done_buffer).unwrap();
    let done_string = String::from_utf8_lossy(&done_buffer).to_string().trim_matches(char::from(0)).to_string();
    if DONE.eq(&done_string) {
        bar.finish();
        println!("Received file successfully");
    } else {
        println!("Error in receiving the entire file");
    }
}
