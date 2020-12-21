use serde::{Deserialize,Serialize};
use serde_json;

#[derive(Debug,Deserialize,Serialize)]
pub struct UserInfo {
    #[serde(default="def_display")]
    pub display: String,
    #[serde(default="def_channel")]
    pub channel: String,
    #[serde(default="def_admin")]
    pub admin: bool,
    #[serde(default="def_err")]
    pub err: String,
}

fn def_display() -> String { "[no-name]".to_string() }
fn def_channel() -> String { "no-channel".to_string() }
fn def_admin() -> bool { false }
fn def_err() -> String { "".to_string() }

impl UserInfo {
    pub fn new(info:&str) -> Option<UserInfo> {
        match serde_json::from_str(info) {
            Ok(u) => Some(u),
            Err(_) => None,
        }

    }
}