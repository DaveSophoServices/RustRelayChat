use std::sync::{mpsc};
use std::net::{SocketAddr};
use chrono::prelude::*;

use log::{debug};

pub struct LogMessage {
    user: String,
    addr: SocketAddr,
    channel: String,
    datetime: DateTime<Utc>,
    message: String,
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
