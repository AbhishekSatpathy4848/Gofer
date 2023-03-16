//single threaded server for now
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, Arc, mpsc};
use std::thread::sleep;
// use log;
use std::{thread, fs};
use std::fs::File;
use std::io::{Write, Read};
use std::time::Duration;
mod handler;

fn main() {
    let path = "./src/recv.txt";
    fs::write(path, "");
    let addr = "127.0.0.1:7878";
    let server = TcpListener::bind(&addr).unwrap();
    let mut client_count = 0;

    // log::info!("Listening on {addr}");

    let (mut tx,rx) = spmc::channel();
    let (mut tx_server,rx_client_1) = spmc::channel();
    
    let (mut tx_client_2,rx_server) = mpsc::channel();

    // let mut public_key_client = Vec::new();
    let public_key_client_mutex = Arc::new(Mutex::new(Vec::new()));
    let passphrase_client_mutex = Arc::new(Mutex::new(String::from("")));

    let public_key_client_2_mutex = Arc::new(Mutex::new(Vec::new()));

    let test_mutex = Arc::new(Mutex::new(1));

    for stream in server.incoming() {
        match stream {
            Ok(mut stream) => {
                
                // let mut passphrase = [0u8;128];
                // // stream.set_read_timeout(None).expect("Error in setting read timeout");
                // stream.read(&mut passphrase).unwrap();
                // // while(buf.len() != 0){
                // //     stream.read(&mut buf).unwrap();
                //     // a = String::from_utf8_lossy(&buf).to_string().trim().to_string();
                //     // if(a.as_str().eq(&recv_string)){
                //         // break;
                //     // }
                // // }
                // print!("Password is {}",String::from_utf8_lossy(&passphrase).to_string());

                // stream.write_all("ACK".as_bytes()).unwrap();
                // stream.flush().unwrap();

                // // stream.read(&mut public_key_client_1).unwrap();
                
                // //33 is the size of the public key vector and yeah I counted that manually
                // let mut public_key_client_1 = [0u8; 33]; 
    
                // //read file size
                // stream.read(&mut public_key_client_1).unwrap();
                // println!("{:?}", public_key_client_1);
                // let msg_len_str = decode_message_size(&mut r);
                // println!("{:?}", msg_len_str);
                // r.to_vec();
                


                client_count+=1;
                if client_count != 1 {tx.send(client_count).unwrap()};
                println!("New connection: {}", stream.peer_addr().unwrap());

                let rx = rx.clone();
                let tx_client_2 = tx_client_2.clone();
                let rx_client_1 = rx_client_1.clone();
                
                let passphrase_client_mutex_clone = Arc::clone(&passphrase_client_mutex);
                let public_key_client_mutex_clone = Arc::clone(&public_key_client_mutex);
                let public_key_client_2_mutex_clone = Arc::clone(&public_key_client_2_mutex);
                // let (mut tx_client_2,rx_client_1) = spmc::channel();
                let test_mutex_clone = Arc::clone(&test_mutex);
                println!("before thread");
                //thread
                thread::spawn(move|| {
                    // connection succeeded
                    
                    if client_count == 2 {

                    //     // stream.write_all(String::from("Recv").as_bytes()).unwrap();
                    //     // send_file(stream);
                    //     println!("client count 2");
                        let mut temp = [0u8;33];
                        stream.read(&mut temp).unwrap();
                        println!("Message/Public Key {:?}",temp);
                        let passphrase = passphrase_client_mutex_clone.lock().unwrap();
                        println!("Passphrase is {}",passphrase);
                    //     // let s_clone = s.clone();
                        stream.write(passphrase.as_bytes()).unwrap();
                        stream.flush();
                        println!("Sent passphrase!!");
                        

                        let public_key_client = public_key_client_mutex_clone.lock().unwrap();
                        temp = [0u8;33];
                        println!("Sending {:?}",public_key_client);
                        stream.write(&public_key_client).unwrap();
                        stream.flush();
                        println!("Sent message");
                        
                        //waiting for client 2 to send its public key
                        // let mut public_key_client_2 = [0u8; 33];
                        // stream.read(&mut public_key_client_2).unwrap();
                        // println!("Public key of client 2 is {:?}", public_key_client_2);
                        // stream.flush();
                        // let mut public_key_client_2 = [0u8, 33]; 
                        // stream.read(&mut public_key_client_2).unwrap();
                        // println!("Yah");
                        // println!("Public key is {:?}", public_key_client_2);
                        temp = [0u8;33];
                        
                        // stream.read(&mut temp).unwrap();
                        // println!("Got something{:?}",String::from_utf8_lossy(&temp).to_string());
                        // let mut buffer_message = [0u8; 33]; 
                        stream.read(&mut temp).unwrap();
                        println!("Public key of client 2 is {:?}", temp);
                        {
                            let mut public_key_client_2 = public_key_client_2_mutex_clone.lock().unwrap();
                            *public_key_client_2 = temp.to_vec();
                        }
                        tx_client_2.send("Start").unwrap();
                        // return;
                    }
                    
                    //key exchange start
                    let mut passphrase = [0u8;128];
                    stream.read(&mut passphrase).unwrap();
                    print!("Passphrase is {}",String::from_utf8_lossy(&passphrase).to_string());
                    //attaching explicit scope ensures that the mutex is unlocked immediately after its usage
                    {
                        let mut mutex_guard_passphrase = passphrase_client_mutex_clone.lock().unwrap();
                        *mutex_guard_passphrase = String::from_utf8_lossy(&passphrase).to_string();
                    }
                    //TODO:write code to check ACK in client
                    stream.write_all("ACK".as_bytes()).unwrap();
                    stream.flush().unwrap();
                    //33 is the size of the public key vector 
                    let mut public_key_client_1 = [0u8; 33]; 
                    stream.read(&mut public_key_client_1).unwrap();
                    {
                        let mut mutex_guard_public_key = public_key_client_mutex_clone.lock().unwrap();
                        *mutex_guard_public_key = public_key_client_1.to_vec();
                    }
                    // public_key_client = public_key_client_1.to_vec();
                    println!("Public key is {:?}", public_key_client_1);
                    println!("beforeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
                    rx_client_1.recv().unwrap();
                    // sleep(Duration::from_secs(20));
                    println!("afterrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr");
                    {
                        let public_key_client_2 = public_key_client_2_mutex_clone.lock().unwrap();
                        stream.write(&public_key_client_2).unwrap();
                    }                    
                    //key exchange close
                    
                    // while rx.recv().unwrap() != 2 {};
                    // stream.write_all(String::from("Send").as_bytes()).unwrap();
                    // handler::handle_incoming_conn(stream);
                }
            );
            if client_count == 2{
                println!("Waiting for client 2 to signal");
                let received_string = rx_server.recv().unwrap();
                println!("Received signal from client 2");
                tx_server.send(received_string).unwrap();
                println!("Send signal from client 1");
            }
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