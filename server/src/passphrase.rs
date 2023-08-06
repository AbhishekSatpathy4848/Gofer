use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub fn get_passphrase(id: i32) -> Result<String,String>{
    if id.is_negative() {
        return Err("id is negative".to_string());
    }
    let file = File::open("/app/src/passphrase.txt");
    let mut passphrases:Vec<String> = Vec::new();
    match file{
        Ok(f) => {
            for line in BufReader::new(f).lines(){
                passphrases.push(line.unwrap());
            }
        }
        Err(e) => {
            println!("error opening file {}",e.to_string());
            return Err(e.to_string());
        }
    }
    if passphrases.len()<=id as usize {
        return Err("id out of bounds".to_string());
    }
    return  Ok(passphrases.get(id as usize).unwrap().to_string());
}

pub fn get_number_of_passphrases() -> Result<usize,String>{
    let file = File::open("/app/src/passphrase.txt");
    match file{
        Ok(f) => {
            let num =  BufReader::new(f).lines().count();
            return Ok(num);
        }
        Err(e) => {
            println!("error opening file {}",e.to_string());
            return Err(e.to_string());
        }
    }
}

