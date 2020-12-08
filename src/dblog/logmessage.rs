use std::net::{SocketAddr};
use chrono::prelude::*;

pub struct LogMessage {
    pub user: String,
    pub addr: SocketAddr,
    pub channel: String,
    pub datetime: DateTime<Utc>,
    pub message: String,
}

pub fn new(user:String, addr: SocketAddr, channel: String, message:String) -> LogMessage {
    LogMessage {
	user,
	addr,
	channel,
	datetime: Utc::now(),
	message,
    }
}
