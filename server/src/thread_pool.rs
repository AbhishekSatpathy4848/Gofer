use std::{thread, sync::{mpsc::{self, Receiver}, Mutex, Arc}, collections::HashMap, net::TcpStream, time::Duration};
use std::io::{Read, Write};
use std::str;

const ACK: &str = "ack "; //ensure this is 4 bytes long
const DONE: &str = "done";

pub struct ThreadPool{
    threads: Vec<Worker>,
    sender: mpsc::Sender<Message>,
    passphrase_map: Arc<Mutex<HashMap<String,usize>>>,
    thread_communication_channels: Vec<mpsc::Sender<TcpStream>>,
}

enum Message{
    NewJob(TcpStream, String),
    Terminate,
}

impl ThreadPool{
    /// Create a new ThreadPool
    /// The size is the number of threads in the pool
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> ThreadPool{ 
        assert!(size > 0);

        let mut workers = Vec::with_capacity(size);

        let passphrase_map: Arc<Mutex<HashMap<String,usize>>> = Arc::new(Mutex::new(HashMap::new()));

        let (sender, receiver) = mpsc::channel();

        let common_receiver = Arc::new(Mutex::new(receiver));

        let mut thread_communication_channels: Vec<mpsc::Sender<TcpStream>> = Vec::new();

        //check round robin ig
        for id in 0..size{

            //each thread has two types of receivers
            //1. all workers read from the one common receiver from where they obtain jobs from the main thread
            //2. each thread has its own reciever via which it communicates with the main thread
            
            let (sender, individual_reciever)= mpsc::channel();
            thread_communication_channels.push(sender);

            let worker = Worker::new(id, Arc::clone(&common_receiver), Arc::clone(&passphrase_map), individual_reciever);
            workers.push(worker);

        }

        ThreadPool{
            threads: workers,
            sender,
            passphrase_map: passphrase_map,
            thread_communication_channels: thread_communication_channels,
        }
    }
    pub fn execute(&self, stream:TcpStream, passphrase: String)
    {
        self.sender.send(Message::NewJob(stream,passphrase)).unwrap();
    }
    pub fn dispatch(&self, stream:TcpStream, mut passphrase: String)
    {
        passphrase = passphrase.trim_matches(char::from(0)).to_string();
        let s = self.passphrase_map.lock();
        let binding = s.unwrap();
        let thread_id = binding.get(&passphrase).cloned(); // clone the value so we can release the lock
        drop(binding);

        match thread_id {
            Some(id) => {
                println!("Sending stream to worker {:?}", id);
                self.thread_communication_channels[id].send(stream).unwrap();
            }
            None => {
                println!("No thread found for passphrase {}", passphrase);
            }
        }
    }
}

impl Drop for ThreadPool{
    fn drop(&mut self){
        println!("Sending terminate message to all workers.");
        for _ in &mut self.threads{
            self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers.");
        for worker in &mut self.threads{
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
                println!("Worker {} is down", worker.id);
            }
        } 
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker{
    fn new(id: usize,job_receiver: Arc<Mutex<Receiver<Message>>>, passphrase_map: Arc<Mutex<HashMap<String, usize>>>, receiver: Receiver<TcpStream>) -> Worker{
        let thread = thread::spawn(move || loop{
            let message = job_receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(mut sender_stream, passphrase) => {

                    println!("Worker {} received a job; Executing...", id);


                    let mut passhrase_mutex_guard = passphrase_map.lock().unwrap();
                    let copy_passphrase = passphrase.clone();
                    passhrase_mutex_guard.insert(passphrase, id);
                    drop(passhrase_mutex_guard);
                    
                    
                    println!("Waiting for client 2 from worker {}...", id);
                    let mut receiver_stream = receiver.recv().unwrap();
                    println!("Client 2 connected to worker {}!!", id);

                    sender_stream.write_all(ACK.as_bytes()).unwrap();

                    
                    //waiting for public key of sender
                    let mut public_key_sender_buffer = [0u8;64];
                    sender_stream.read(&mut public_key_sender_buffer).unwrap();
                    println!("Public key of sender {:?}",public_key_sender_buffer);
                    println!();

                    //sending public key of sender to receiver
                    receiver_stream.write_all(&public_key_sender_buffer).unwrap();

                    //waiting for public key of receiver
                    let mut public_key_receiver_buffer = [0u8;64];
                    receiver_stream.read(&mut public_key_receiver_buffer).unwrap();
                    println!("Public key of receiver {:?}",public_key_receiver_buffer);
                    println!();

                    //sending public key of receiver to sender
                    sender_stream.write_all(&public_key_receiver_buffer).unwrap();

                    //waiting for sender to send file size
                    let mut file_size_buffer = [0u8;16];
                    sender_stream.read(&mut file_size_buffer).unwrap();
                    println!("File size {:?}",file_size_buffer);

                    println!("File size {:?}",decode_message_size(&mut file_size_buffer));
                    println!();

                    //send file size to receiver
                    receiver_stream.write_all(&file_size_buffer).unwrap();


                    loop {
                        //waiting for ACK from receiver
                        // println!("Waiting for ACK from receiver");
                        let mut ack_buffer = [0u8;3];
                        receiver_stream.read(&mut ack_buffer).unwrap();


                        //sending ACK to the sender to send the next block
                        // println!("Sending ACK to sender to send the next block");
                        sender_stream.write_all(ACK.as_bytes()).unwrap();

                        //waiting for file from sender and send to receiver
                        let mut file_buffer = [0u8;16];
                        
                        sender_stream.read(&mut file_buffer).unwrap();
                        // println!("received file from sender and sending to receiver");
                        receiver_stream.write_all(&file_buffer).unwrap();
                        // println!("sent file to receiver");
                        // println!("{:?}",file_buffer);
                        
                        //check if done and signal the receiver as well
                        let ack_string = String::from_utf8_lossy(&file_buffer).to_string().trim_matches(char::from(0)).to_string();
                        
                        if DONE.eq(&ack_string) {
                            println!("Done sending file");
                            let mut file_buffer = [0u8;16];
                            receiver_stream.read(&mut file_buffer);
                            break;
                        }
                    }

                    let mut passhrase_mutex_guard = passphrase_map.lock().unwrap();
                    passhrase_mutex_guard.remove(&copy_passphrase);
                }
                Message::Terminate =>{
                    println!("Worker {} was told to terminate.", id);
                    break;
                }
            }
        });
        Worker{
            id,
            thread:Some(thread),
        }
    }
}

fn decode_message_size(mut buffer: &mut [u8]) -> String {
    let msg_len_slice: &str = str::from_utf8(&mut buffer).unwrap();
    let mut msg_len_str = msg_len_slice.to_string();
    println!("msg_len_str: {:?}", msg_len_str);
    let mut numeric_chars = 0;
    for c in msg_len_str.chars() {
        if c.is_numeric() == true {
            numeric_chars = numeric_chars + 1;
        }
    }
    msg_len_str.truncate(numeric_chars);
    msg_len_str
}