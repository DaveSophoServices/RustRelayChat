use std::net::{SocketAddr};
use chrono::prelude::*;
use crate::userinfo::UserInfo;

pub struct LogMessage {
    pub userinfo: UserInfo,
    pub addr: SocketAddr,
    pub channel: String,
    pub datetime: DateTime<Utc>,
    pub message: String,
}

pub fn new(userinfo:UserInfo, addr: SocketAddr, channel: String, message:String) -> LogMessage {
    LogMessage {
        userinfo,
        addr,
        channel,
        datetime: Utc::now(),
        message,
    }
}
