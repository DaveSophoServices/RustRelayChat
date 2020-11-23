use serde::{Deserialize,Serialize};
use serde_json;

#[derive(Debug,Deserialize,Serialize)]
pub struct Config {
    #[serde(default="def_port")]
    pub port:i32,
    #[serde(default="def_auto_create_rooms")]
    pub auto_create_rooms:bool,
}

pub fn parse_json(json:&str) -> Config {
    serde_json::from_str(json).expect("problem reading JSON config file")
}

pub fn default() -> Config {
    Config {
	port: def_port(),
	auto_create_rooms: def_auto_create_rooms(),
    }
}

fn def_port() -> i32 {
    9001
}

fn def_auto_create_rooms() -> bool {
    true
}

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
}
