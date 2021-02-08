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
		let s = Server {
			shutdown: Arc::new(RwLock::new(0)),
			channels: Arc::new(RwLock::new(HashMap::new())),
			dblogger: Arc::new(dblog::new(config.clone())),
			config,
		};
		if s.config.startup_rooms.len() > 0 {
			let rooms = s.config.startup_rooms.clone();
			for ch in rooms {
				s.create_channel_server(ch);
			}
		}
		return s;
	}
	
	pub fn shutdown_ref(&self) -> Arc<RwLock<u32>> {
		self.shutdown.clone()
	}
	
	pub fn get(&self, ws_hdr:Arc<RwLock<WebsocketHeaders>>) -> Option<ChannelServer> {
		let uri = match &ws_hdr.read().unwrap().uri {
			Some(x) => x.to_string(),
			None => String::from(""),
		};
		let ret:Option<ChannelServer>;
		match self.channels.read() {
			Ok(channels) => {
				ret = match channels.get(&uri) {
					Some(x) => Some(x.clone()),
					None => None
				}
			}
			Err(_) => panic!("failed to get read access to channel list!"),
		}
		if self.config.auto_create_rooms && ret.is_none() {
			self.create_channel_server(uri)
		} else {
			ret
		}
	}
	
	pub fn create_channel_server(&self, name:String) -> Option<ChannelServer> {
		match self.channels.write() {
			Ok(mut channels) => {
				debug!("Creating new channel_server: {}", name);
				let name_key = name.clone();
				channels.insert(name_key, channel_server::new(self.shutdown.clone(), &name));
				Some(channels.get(&name).unwrap().clone())
			},
			Err(_) => panic!("failed to get write access to channel list!"),
		}
	}

	pub fn logger_channel(&self) -> Option<mpsc::Sender<dblog::logmessage::LogMessage>> {
		self.dblogger.get_sender()
		
	}
	pub fn get_secret_key(&self) -> &str {
		return &self.config.seckey;
	}
}
