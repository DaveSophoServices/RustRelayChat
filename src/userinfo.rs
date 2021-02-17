use serde::{Deserialize,Serialize};
use serde_json;

#[derive(Debug,Deserialize,Serialize,Clone)]
pub struct UserInfo {
    #[serde(default="def_username")]
    pub username: String,
    #[serde(default="def_display")]
    pub display: String,
    #[serde(default="def_first_last")]
    pub first_last: String,
    #[serde(default="def_channel")]
    pub channel: String,
    #[serde(default="def_admin")]
    pub admin: bool,
    #[serde(default="def_err")]
    pub err: String,
}

fn def_username() -> String { "[no-username]".to_string() }
fn def_display() -> String { "[no-name]".to_string() }
fn def_first_last() -> String { "[no-first-last-set]".to_string() }
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
    pub fn blank() -> UserInfo {
        UserInfo {
            username: def_username(),
            display: def_display(),
            first_last: def_first_last(),
            channel: def_channel(),
            admin: def_admin(),
            err: def_err(),
        }
    }
}