use serde::{Deserialize,Serialize};
use serde_json;
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use log::warn;

#[derive(Debug,Deserialize,Serialize)]
pub struct Config {
    #[serde(default="def_port")]
    pub port:i32,
    #[serde(default="def_auto_create_rooms")]
    pub auto_create_rooms:bool,
    #[serde(default="def_dbuser")]    
    pub dbuser:String,
    #[serde(default="def_dbpass")]
    pub dbpass:String,
    #[serde(default="def_dbhost")]
    pub dbhost:String,
    #[serde(default="def_dbport")]
    pub dbport:i32,
    #[serde(default="def_certkey")]
    pub certkey:String,
    #[serde(default="def_certkeypassword")]
    pub certkeypassword:String,
    #[serde(default="def_dbname")]
    pub dbname:String,
    #[serde(default="def_seckey")]
    pub seckey:String,
    #[serde(default="def_startup_rooms")]
    pub startup_rooms:Vec<String>,
}

pub fn parse_json(json:&str) -> Config {
    serde_json::from_str(json).expect("problem reading JSON config file")
}

pub fn default() -> Config {
    Config {
	port: def_port(),
	auto_create_rooms: def_auto_create_rooms(),
	dbuser: def_dbuser(),
	dbpass: def_dbpass(),
	dbhost: def_dbhost(),
	dbport: def_dbport(),
    dbname: def_dbname(),
    certkey: def_certkey(),
    certkeypassword: def_certkeypassword(),
    seckey: def_seckey(),
    startup_rooms: def_startup_rooms(),
    }
}

fn def_port() -> i32 { 9001 }

fn def_auto_create_rooms() -> bool { true }

fn def_dbuser() -> String { "".to_string() }

fn def_dbpass() -> String { "".to_string() }

fn def_dbhost() -> String { "".to_string() }

fn def_dbport() -> i32 { -1 }

fn def_dbname() -> String { "".to_string() }

fn def_certkey() -> String { "".to_string() }

fn def_certkeypassword() -> String { "".to_string() }

fn def_seckey() -> String { 
    let mut rnd = ChaCha20Rng::from_entropy(); 
    let x:u32 = rnd.gen(); 
    warn!("Using random value for seckey."); 
    format!("{}",x) 
}

fn def_startup_rooms() -> Vec<String> { vec!() } // empty list

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_parses() {
	let c = parse_json(r#"{"port": 9001,"auto_create_rooms": true}"#);
	assert_eq!(c.port, 9001, "Port is 9001");
	assert_eq!(c.auto_create_rooms, true);
    }

    #[test]
    fn config_partial() {
	let c = parse_json("{}");
    }

    #[test]
    fn config_rooms() {
        let c = parse_json(r#"{"startup_rooms": [ "room_a", "room_b" ] }"#);
        assert_eq!(c.startup_rooms.len(), 2, "2 entries in startup rooms");
        let c = parse_json(r#"{ }"#);
        assert_eq!(c.startup_rooms.len(), 0, "blank list of startup rooms");
    }
}
