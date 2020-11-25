use std::net::SocketAddr;
use std::sync::{Arc,RwLock};
use tungstenite::WebSocket;

use crate::server::channel_server::ChannelServer;
use crate::stats::Stats;

pub struct Client {
    name: String,
    addr: SocketAddr,
    websocket: Arc<WebSocket>,
    ch: Arc<ChannelServer>,
    pair_shutdown: i32,
    stats: Arc<Stats>,
}
