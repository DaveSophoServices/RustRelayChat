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
}

struct ChannelList {
    list: HashMap<String,(Arc<ChannelServer>, Arc<mpsc::Sender<tungstenite::Message>>)>,
}

static CHANNELS: RwLock<ChannelList> = RwLock::new(
    ChannelList { list: HashMap::new(), });

// Server holds a map of channel strings to channel servers
impl Server {
    pub fn new() -> Server {
	Server {
	    shutdown: Arc::new(RwLock::new(0)),
	}
    }

    pub fn shutdown_ref(&self) -> Arc<RwLock<u32>> {
	self.shutdown.clone()
    }
    
    pub fn get(&self, ws_hdr:Arc<RwLock<WebsocketHeaders>>) -> Arc<ChannelServer>{
	let uri = match ws_hdr.read().unwrap().uri {
	    Some(x) => x.to_string(),
	    None => String::from(""),
	};
	let mut ret: Option<&Arc<ChannelServer>>;
	let tx: &Arc<mpsc::Sender<tungstenite::Message>>;
	if let Ok(channels) = CHANNELS.read() {
	    if let Some((r,t)) = channels.list.get(&uri) {
		ret = Some(r);
		tx = t;
	    }
	}
	match ret {
	    Some(x) => x.clone(),
	    None => {
		debug!("Creating new channel_server: {}", uri);
		match CHANNELS.write() {
		    Ok(channels) => {
			channels.list.insert(uri, channel_server::new(self.shutdown.clone(), uri));
			channels.list.get(&uri).unwrap().0
		    },
		    Err(_) => panic!("server::get : CHANNELS RwLock failed to write"),
		}
	    }
	}
    }
}

pub fn sendrecv() {
}
