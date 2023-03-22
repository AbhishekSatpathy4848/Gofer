//single threaded server for now
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
// use std::thread::sleep;
// use log;
use std::fs::File;
use std::io::{Read, Write};
use std::{fs, thread};
// use std::time::Duration;
mod handler;

fn main() {
    let path = "/Users/abhisheksatpathy/gofer/server/src/recv.txt";
    fs::write(path, "").unwrap();
    let addr = "192.168.130.182:7878";
    let server = TcpListener::bind(&addr).unwrap();
    let mut client_count = 0;

    // log::info!("Listening on {addr}");

    // let (mut tx,rx) = spmc::channel();

    let (mut tx_server_client_1, rx_client_1_server) = spmc::channel();
    let (tx_client_2_server, rx_server_client_2) = mpsc::channel();

    let (tx_client_1_server, rx_server_client_1) = mpsc::channel();
    let (mut tx_server_client_2, rx_client_2_server) = spmc::channel();

    // let mut public_key_client = Vec::new();
    // let public_key_client_mutex = Arc::new(Mutex::new(Vec::new()));
    let passphrase_client_mutex = Arc::new(Mutex::new(String::from("")));

    // let public_key_client_2_mutex = Arc::new(Mutex::new(Vec::new()));

    // let test_mutex = Arc::new(Mutex::new(1));
    // let mut send = vec![];

    for stream in server.incoming() {
        match stream {
            Ok(mut stream) => {
                // let s=stream;
                // send.push(&stream);

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

                client_count += 1;
                // if client_count != 1 {tx.send(client_count).unwrap()};
                println!("New connection: {}", stream.peer_addr().unwrap());

                // let rx = rx.clone();
                let tx_client_2_server = tx_client_2_server.clone();
                let rx_client_1_server = rx_client_1_server.clone();
                let rx_client_2_server = rx_client_2_server.clone();
                let tx_client_1_server = tx_client_1_server.clone();
                // let rx_server_client_1 = rx_server_client_1.clone();

                let passphrase_client_mutex_clone = Arc::clone(&passphrase_client_mutex);
                // let public_key_client_mutex_clone = Arc::clone(&public_key_client_mutex);
                // let public_key_client_2_mutex_clone = Arc::clone(&public_key_client_2_mutex);
                // let (mut tx_client_2_server,rx_client_1_server) = spmc::channel();
                // let test_mutex_clone = Arc::clone(&test_mutex);
                println!("before thread");
                //thread
                thread::spawn(move || {
                    // connection succeeded

                    if client_count == 2 {
                        // println!("client count 2");
                        let mut temp = [0u8; 128];
                        //waits for clients ack
                        stream.read(&mut temp).unwrap(); //should have passphrase in temp
                        println!(
                            "Received passphrase {}",
                            String::from_utf8_lossy(&temp).to_string()
                        );
                        let mut str:String;
                        let mut str2:String;
                            {let passphrase = passphrase_client_mutex_clone.lock().unwrap();
                            str = String::from_utf8_lossy(&temp).to_string();
                            str = str.trim().to_string();
                            println!("{:?}",str.as_bytes());
                            str2 = passphrase.clone();
                            str2 = str2.trim().to_string();
                            println!("{:?}",str2.as_bytes());
                            }
                            

                            if str.eq(&str2) {
                                println!("Passphrase matched");
                                stream.write_all("ACK".as_bytes()).unwrap();
                                // let passphrase = passphrase_client_mutex_clone.lock().unwrap();
                                // println!("Passphrase is {}", passphrase);
                                // stream.write_all(passphrase.as_bytes()).unwrap();
                                // println!("Sent passphrase!!");
                                let mut temp = [0u8; 33];
                                tx_client_2_server.send(temp).unwrap();
                                temp = rx_client_2_server.recv().unwrap();
                                println!("Client-1 Public Key {:?}", temp);
                                // println!("Sending {:?}",temp);
                                stream.write_all(&temp).unwrap();
                                println!("Sent message to client");

                                //waiting for client 2 to send its public key
                                let mut public_key_client_2 = [0u8; 33];
                                stream.read(&mut public_key_client_2).unwrap();
                                println!("Public key of client 2 is {:?}", public_key_client_2);

                                //using mutex start
                                // stream.read(&mut temp).unwrap();
                                // println!("Public key of client 2 is {:?}", temp);
                                // {
                                //     let mut public_key_client_2 = public_key_client_2_mutex_clone.lock().unwrap();
                                //     *public_key_client_2 = temp.to_vec();
                                // }
                                // tx_client_2_server.send("Start").unwrap();
                                //using mutex end

                                println!("Sending client 2 public key\n");
                                tx_client_2_server.send(public_key_client_2).unwrap();
                                println!("Sent public key of client 2 to server\n");
                                sleep(std::time::Duration::from_secs(2));
                                send_file(stream);
                                client_count = 0;
                            } else {
                                println!("Passphrase did not match");
                                stream.write_all("NACK".as_bytes()).unwrap();
                                // sleep(std::time::Duration::from_secs(2));
                                // return;
                        }
                    } else if client_count == 1 {
                        //key exchange start
                        let mut passphrase = [0u8; 128];
                        stream.read(&mut passphrase).unwrap();
                        print!(
                            "Passphrase is {}",
                            String::from_utf8_lossy(&passphrase).to_string()
                        );
                        //attaching explicit scope ensures that the mutex is unlocked immediately after its usage
                        {
                            let mut mutex_guard_passphrase =
                                passphrase_client_mutex_clone.lock().unwrap();
                            *mutex_guard_passphrase =
                                String::from_utf8_lossy(&passphrase).to_string();
                        }
                        //TODO:write code to check ACK in client
                        stream.write_all("ACK".as_bytes()).unwrap();
                        stream.flush().unwrap();
                        //33 is the size of the public key vector
                        let mut public_key_client_1 = [0u8; 33];
                        stream.read(&mut public_key_client_1).unwrap();

                        println!(
                            "Received passphrase {:?} and public key of client 1 {:?}\n",
                            passphrase, public_key_client_1
                        );

                        //using mutex
                        // {
                        //     let mut mutex_guard_public_key = public_key_client_mutex_clone.lock().unwrap();
                        //     *mutex_guard_public_key = public_key_client_1.to_vec();
                        // }

                        //using message passing only for public key
                        rx_client_1_server.recv().unwrap();
                        println!("Received message that client 2 is ready to receive public key\n");
                        tx_client_1_server.send(public_key_client_1).unwrap();

                        //using mutex end
                        let mut public_key_client_2 = [0u8; 33];
                        println!("Waiting.....");
                        public_key_client_2 = rx_client_1_server.recv().unwrap();
                        println!("Client {:?}", public_key_client_2);
                        stream.write_all(&public_key_client_2).unwrap();

                        // while rx.recv().unwrap() != 2 {};
                        // stream.write_all(String::from("Send").as_bytes()).unwrap();
                        handler::handle_incoming_conn(stream);
                    }
                });

                //inside main thread of the server
                if client_count == 2 {
                    //signalling client 1 thread that client 2 has connected and public key can be sent
                    let mut temp = [0u8; 33];
                    temp = rx_server_client_2.recv().unwrap();
                    tx_server_client_1.send(temp).unwrap();

                    let public_key_client_1 = rx_server_client_1.recv().unwrap();
                    println!(
                        "Server has received public key of client_1 {:?}\n",
                        public_key_client_1
                    );
                    tx_server_client_2.send(public_key_client_1).unwrap();
                    // sleep(Duration::from_secs(20));
                    // println!("Waiting for client 2s public key\n");
                    // let mut received_string = [0u8;33];
                    let received_string = rx_server_client_2.recv().unwrap();
                    println!("Received public key from client 2 {:?}\n", received_string);
                    tx_server_client_1.send(received_string).unwrap();
                    println!("Send public key of client 2 to client 1\n");
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    println!("Done");

    drop(server);
}

fn send_file(mut stream: TcpStream) {
    let path = "/Users/abhisheksatpathy/gofer/server/src/recv.txt";
    let mut file_size = fs::metadata(path).unwrap().len();
    while file_size == 0 {
        file_size = fs::metadata(path).unwrap().len();
    }
    print!("File size : {}\n", file_size);

    // let file_name = "recv.txt";
    // let mut fullname = String::from("./src/");
    // fullname.push_str(file_name);
    // println!("FULLPATH: {:?}", fullname);

    let mut remaining_data = file_size as i32;
    println!("Bytes:  {:?}",remaining_data.to_string().as_bytes());
    stream
        .write_all(remaining_data.to_string().as_bytes())
        .unwrap();

    let mut buf = [0u8; 16];
    let mut file = File::open(path).unwrap();

    while remaining_data != 0 {
        if remaining_data >= 16 {
            //read slab from file
            let file_slab = file.read(&mut buf);
            match file_slab {
                Ok(n) => {
                    stream.write_all(&buf).unwrap();
                    println!("sent {} file bytes (big)", n);
                    remaining_data = remaining_data - n as i32;
                }
                _ => {}
            }
        } else {
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
