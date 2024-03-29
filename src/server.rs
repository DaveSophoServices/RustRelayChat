use std::sync::{Arc,RwLock,mpsc};
use std::collections::HashMap;
use log::{debug};
use native_tls::{Identity, TlsAcceptor};

pub mod channel_server;
use channel_server::ChannelServer; 
use super::websocket_headers::WebsocketHeaders;
use crate::config;
#[cfg(feature="dblog")]
use crate::dblog;
use log::warn;

#[derive(Clone)]
pub struct Server {
	shutdown: Arc<RwLock<u32>>,
	channels: Arc<RwLock<HashMap<String,ChannelServer>>>,
	config: Arc<config::Config>,
	#[cfg(feature="dblog")]
	dblogger: Arc<dblog::DBLog>,
	acceptor: Option<Arc<native_tls::TlsAcceptor>>,
}

// Server holds a map of channel strings to channel servers
impl Server {
	pub fn new(config:Arc<config::Config>, identity:Vec<u8>) -> Server {

		let acceptor =
			if identity.len() > 0 {
				let identity = match Identity::from_pkcs12(&identity, &config.certkeypassword) {
					Ok(i) => i,
					Err(why) => {
						warn!("could not extract PKCS12 format key from {}.", config.certkey);
						warn!("see https://docs.rs/native-tls/0.2.8/native_tls/struct.Identity.html for information regarding creating a pkcs12 format export.");
						warn!(" eg: openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem -certfile chain_certs.pem");
			
						panic!("could not extract PKCS12 format key from {}: {}", config.certkey, why)},
				};
				match TlsAcceptor::new(identity) {
					Ok(a) => Some(Arc::new(a)),
					Err(why) => panic!("could not create TlsAcceptor for identity read from {}", config.certkey)
				}
			} else {
				None
			};

		let s = Server {
			shutdown: Arc::new(RwLock::new(0)),
			channels: Arc::new(RwLock::new(HashMap::new())),
			#[cfg(feature="dblog")]
			dblogger: Arc::new(dblog::new(config.clone())),
			config,
			acceptor: None,
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

	#[cfg(feature="dblog")]
	pub fn logger_channel(&self) -> Option<mpsc::Sender<dblog::logmessage::LogMessage>> {
		self.dblogger.get_sender()
		
	}
	pub fn get_secret_key(&self) -> &str {
		return &self.config.seckey;
	}
	pub fn has_tls(&self) -> bool {
		self.acceptor.is_some()
	}
	pub fn negotiate_tls(&self, stream:std::net::TcpStream) -> Result<native_tls::TlsStream<std::net::TcpStream>, native_tls::HandshakeError<std::net::TcpStream>> {
		let acceptor = self.acceptor.clone().unwrap();
		acceptor.accept(stream)
	}
}
