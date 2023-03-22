use spake2::{Ed25519Group, Identity, Password, Spake2};
use aes::{self, cipher::{generic_array::GenericArray, KeyInit, BlockEncrypt, BlockDecrypt}, Aes128};
fn main() {
     //client_1   
//    let (s1, outbound_msg) = Spake2::<Ed25519Group>::start_symmetric(
//         &Password::new(b"pass123"),
//         &Identity::new(b"shared id"));

//      // send_server(outbound_msg,"pass123");
//      // forward_client_2(outbound_msg,"pass123");

//      //client_2
//      // outbound_msg , password
//    let (s2, outbound_msg_2) = Spake2::<Ed25519Group>::start_symmetric(
//         &Password::new(b"password"),
//         &Identity::new(b"shared id"));

//      // send_server(outbound_msg_2);
//      // forward_client_1(outbound_msg_2);

//      // s1.finish(msg2);

//     print!("{:?}\n",outbound_msg);
//     print!("{:?}\n",outbound_msg_2);
    
//     print!("{:?}",s2.finish(&outbound_msg).unwrap()); 
//     print!("{:?}",s1.finish(&outbound_msg_2).unwrap());

     // let arr = [1,2,3];
     // let key = Key::from_slice(&arr);
     // let ciplher = ChaCha20Poly1305::new(&key);

     let key = GenericArray::from([0u8;16]); 
     let mut block = GenericArray::from([100u8;16]);
     // let cipher = Aes128::new(&key);

     // let key = GenericArray::from([0u8; 16]);
     // // let mut block = GenericArray::from([42u8; 16]);
     // let block_1 = block.clone();
     // Initialize cipher
     let cipher = Aes128::new(&key);
     println!("Before encryption:{:?}",block);
     cipher.encrypt_block(&mut block);
     println!("After encryption:{:?}",block);
     cipher.decrypt_block(&mut block);
     println!("After decryption:{:?}",block);
    

}

