use std::sync::{Arc,RwLock};
use std::collections::HashMap;
use std::sync::mpsc;
use log::{debug};

mod channel_server;
use channel_server::ChannelServer; 
use super::websocket_headers::WebsocketHeaders;

#[derive(Clone)]
pub struct Server {
    shutdown: Arc<RwLock<u32>>,
    channels: Arc<RwLock<HashMap<String,ChannelServer>>>,
}

// Server holds a map of channel strings to channel servers
impl Server {
    pub fn new() -> Server {
	Server {
	    shutdown: Arc::new(RwLock::new(0)),
	    channels: Arc::new(RwLock::new(HashMap::new())),
	}
    }

    pub fn shutdown_ref(&self) -> Arc<RwLock<u32>> {
	self.shutdown.clone()
    }
    
    pub fn get(&self, ws_hdr:Arc<RwLock<WebsocketHeaders>>) -> ChannelServer{
	let uri = match &ws_hdr.read().unwrap().uri {
	    Some(x) => x.to_string(),
	    None => String::from(""),
	};

	match self.channels.write() {
	    Ok(mut channels) => {
		match channels.get(&uri) {
		    Some(x) => x.clone(),
		    None => {
			debug!("Creating new channel_server: {}", uri);
			let uri_key = uri.clone();
			channels.insert(uri_key, channel_server::new(self.shutdown.clone(), &uri));
			channels.get(&uri).unwrap().clone()
		    },
		}
	    },
	    Err(_) => panic!("failed!"),
	}
    }
}

pub fn sendrecv() {
}
