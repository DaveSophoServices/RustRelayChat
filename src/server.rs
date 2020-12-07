use std::sync::{Arc,RwLock,mpsc};
use std::collections::HashMap;
use log::{debug};

pub mod channel_server;
use channel_server::ChannelServer; 
use super::websocket_headers::WebsocketHeaders;
use crate::config;
use crate::dblog;

#[derive(Clone)]
pub struct Server {
    shutdown: Arc<RwLock<u32>>,
    channels: Arc<RwLock<HashMap<String,ChannelServer>>>,
    config: Arc<config::Config>,
    dblogger: Arc<dblog::DBLog>,
}

// Server holds a map of channel strings to channel servers
impl Server {
    pub fn new(config:Arc<config::Config>) -> Server {
	Server {
	    shutdown: Arc::new(RwLock::new(0)),
	    channels: Arc::new(RwLock::new(HashMap::new())),
	    dblogger: Arc::new(dblog::new(config.clone())),
	    config,
	}
    }

    pub fn shutdown_ref(&self) -> Arc<RwLock<u32>> {
	self.shutdown.clone()
    }
    
    pub fn get(&self, ws_hdr:Arc<RwLock<WebsocketHeaders>>) -> Option<ChannelServer> {
	let uri = match &ws_hdr.read().unwrap().uri {
	    Some(x) => x.to_string(),
	    None => String::from(""),
	};

	match self.channels.write() {
	    Ok(mut channels) => {
		match channels.get(&uri) {
		    Some(x) => Some(x.clone()),
		    None => {
			if self.config.auto_create_rooms {
			    debug!("Creating new channel_server: {}", uri);
			    let uri_key = uri.clone();
			    channels.insert(uri_key, channel_server::new(self.shutdown.clone(), &uri));
			    Some(channels.get(&uri).unwrap().clone())
			} else {
			    None
			}
		    },
		}
	    },
	    Err(_) => panic!("failed!"),
	}
    }

    pub fn logger_channel(&self) -> Option<mpsc::Sender<dblog::logmessage::LogMessage>> {
	self.dblogger.get_sender()
	    
    }
}
