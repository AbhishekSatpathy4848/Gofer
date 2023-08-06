use std::collections::HashSet;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;

pub mod passphrase;
pub mod thread_pool;

use passphrase::get_passphrase;
use thread_pool::ThreadPool;


const SEND: &str = "send";
const RECV: &str = "recv";
const ACK: &str = "ack "; //ensure this is 4 bytes long
const NACK: &str = "nack";
const WORKER_THREADS:usize = 3;

enum ClientRole{
    Sender(String, TcpStream),
    Receiver(String, TcpStream),
}

fn main() {
    println!("Server Started!!");
    //current id of the passphrase to be dispatched to the client. Starts from 0.
    let passphrase_id:Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();   
    let pool: ThreadPool = ThreadPool::new(WORKER_THREADS);
    let passphrases_used: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));

        let (sender, receiver) = mpsc::channel();
        thread::spawn(move ||{
            loop{
                let role:ClientRole = receiver.recv().unwrap();
                match role{
                    ClientRole::Sender(passphrase, stream) => {
                        println!("Sender role received");
                        pool.execute(stream, passphrase);
                    }
                    ClientRole::Receiver(passphrase, stream) => {
                        println!("Receiver role received");
                        pool.dispatch(stream, passphrase);
                    }
                }   
                println!();
            }     
        });
  
        for stream in listener.incoming() { 
            println!("Connection established");
            println!();
            //client handler thread
            let passphrase_id_clone = Arc::clone(&passphrase_id);
            let sender_to_server_main = mpsc::Sender::clone(&sender);
            let passphrases_used_clone = Arc::clone(&passphrases_used);
            thread::spawn(move || {
                let mut stream = stream.unwrap();
                let mut buffer = [0; 1024];
                stream.read(&mut buffer).unwrap();
                let string = String::from_utf8_lossy(&buffer);
                println!("Request: {}", string);
                println!();
                let mut choice_string = String::from(&string[0..4]);
                choice_string = choice_string.to_ascii_lowercase();
                
                if SEND.eq(&choice_string) {

                    let mut passphrase_id = passphrase_id_clone.lock().unwrap();
                    let passphrase = get_passphrase(*passphrase_id).unwrap();
                    *passphrase_id = *passphrase_id + 1; 
                    stream.write(passphrase.as_bytes()).unwrap();

                    println!("Sent passphrase {}",passphrase);
                    println!();

                    // println!("{}",passphrase.len());
                    
                    passphrases_used_clone.lock().unwrap().insert(String::clone(&passphrase));
                    
                    sender_to_server_main.send(ClientRole::Sender(passphrase, stream)).unwrap();

                }else if RECV.eq(&choice_string) {
                    let mut buf = [0u8;24];
                    stream.read(&mut buf).unwrap();

                    let mut string = String::from_utf8_lossy(&buf).to_string();
                    string = string.trim_matches(char::from(0)).to_string();
                    

                    if passphrases_used_clone.lock().unwrap().contains(&string) {
                        stream.write(ACK.as_bytes()).unwrap();
                    }else{
                        stream.write(NACK.as_bytes()).unwrap();
                        return;
                    }

                    sender_to_server_main.send(ClientRole::Receiver(String::from_utf8_lossy(&buf).to_string(), stream)).unwrap();
                    
                
                }else{
                    println!("Incorrect choice {}", choice_string);
                    return; 
                }
            });
        }
}

