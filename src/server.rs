use std::sync::{Arc,RwLock};
use std::collections::HashMap;
use log::{debug};

mod channel_server;
use channel_server::ChannelServer; 
use super::websocket_headers::WebsocketHeaders;

#[derive(Clone)]
pub struct Server {
    channels: Arc<HashMap<String,ChannelServer>>,
    shutdown: Arc<RwLock<u32>>,
}


// Server holds a map of channel strings to channel servers
impl Server {
    pub fn new() -> Server {
	Server {
	    channels: Arc<HashMap::new()>,
	    shutdown: Arc::new(RwLock::new(0)),
	}
    }

    pub fn shutdown_ref(&self) -> Arc<RwLock<u32>> {
	self.shutdown.clone()
    }
    
    pub fn get(&self, ws_hdr:Arc<RwLock<WebsocketHeaders>>) -> &ChannelServer{
	let uri = match ws_hdr.read().unwrap().uri {
	    Some(x) => x.to_string(),
	    None => String::from(""),
	}
	match self.channels.get(&uri) {
	    Some(x) => x,
	    None => {
		debug!("Creating new channel_server: {}", uri);
		self.channels.insert(uri, channel_server::new());
		self.channels.get(&uri).unwrap()
	    }
	}
    }
}

pub fn sendrecv() {
}
