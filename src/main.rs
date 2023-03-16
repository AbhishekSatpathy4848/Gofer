use spake2::{Ed25519Group, Identity, Password, Spake2};

fn main() {
     //client_1   
   let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_symmetric(
        &Password::new(b"pass123"),
        &Identity::new(b"shared id"));

     // send_server(outbound_msg,"pass123");
     // forward_client_2(outbound_msg,"pass123");

     //client_2
     // outbound_msg , password
   let (s2, outbound_msg_2) = Spake2::<Ed25519Group>::start_symmetric(
        &Password::new(b"password"),
        &Identity::new(b"shared id"));

     // send_server(outbound_msg_2);
     // forward_client_1(outbound_msg_2);

     // s1.finish(msg2);

    print!("{:?}\n",outbound_msg);
    print!("{:?}\n",outbound_msg_2);
    
    print!("{:?}",s2.finish(&outbound_msg).unwrap()); 
    print!("{:?}",s1.finish(&outbound_msg_2).unwrap());

    

}

