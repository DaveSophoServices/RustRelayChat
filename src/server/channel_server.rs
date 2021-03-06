use std::sync::{mpsc,Arc,Mutex,RwLock};
use std::time::Duration;

use std::thread::spawn;
use log::{debug,error,info,warn};
use tungstenite::Message;
use crate::stats::Stats;
use crate::client::Client;

#[derive(Clone)]
pub struct ChannelServer {
	central_outgoing: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
	clientlist: Arc<Mutex<Vec<Arc<Client>>>>,
	shutdown: Arc<RwLock<u32>>,
	stats: Arc<Stats>,
	channel: String,
	tx: Arc<Mutex<mpsc::Sender<Message>>>,
	rx: Arc<Mutex<mpsc::Receiver<Message>>>,
}

pub fn new(shutdown:std::sync::Arc<std::sync::RwLock<u32>>,
	channel:&String,
) -> ChannelServer  {
	// core's rx and tx
	let (tx,rx) = mpsc::channel();
	
	let ret = ChannelServer {
		central_outgoing: Arc::new(Mutex::new(Vec::new())),
		clientlist: Arc::new(Mutex::new(Vec::new())),
		shutdown,
		stats: Arc::new(Stats::new()),
		channel: channel.clone(),
		tx: Arc::new(Mutex::new(tx)),
		rx: Arc::new(Mutex::new(rx)),		
	};
	
	{
		let mut ret = ret.clone();
		spawn(move || ret.core() );
	}
	return ret;
}
impl ChannelServer {
	pub fn get_name(&self) -> String {
		self.channel.clone()
	}
	
	pub fn get_stats(&self) -> Arc<Stats> {
		return self.stats.clone();
	}
	
	// get_userlist produces a JSON representation 
	// of the current users in this channel
	pub fn get_userlist(&self) -> String {
		let mut s = String::new();
		s.push_str(r#"!*USERLIST {"users":["#);
		
		// iterate the clients and build our string
		if let Ok(list) = self.clientlist.lock() {
			let mut user_list = Vec::new();
			for c in list.iter() {				
				user_list.push(format!(
					r#"{{"user":"{}" }}"#,
					c.get_name()
				));
			}
			s.push_str(&user_list.join(","));
		}
		s.push_str("]}");
		return s;
	}

	pub fn get_tx_rx(&self) -> (mpsc::Sender<tungstenite::Message>,
		mpsc::Receiver<tungstenite::Message>)
	{
		// this pair lets the websocket client receive from us. We return the rx to them, keeping tx in our list
		let (tx,rx) = mpsc::channel();
		if let Ok(mut co_write) = self.central_outgoing.lock() {
			co_write.push(tx);
			self.stats.set_num_clients(co_write.len());
		}
		
		// clone the transmit end of our receiver so that clients can send to us
		(self.tx.lock().unwrap().clone(), rx)
	}

	pub fn add_client(&self, client:Arc<Client>) {
		match self.clientlist.lock() {
			Ok(mut list) => {
				list.push(client);
			},
			Err(e) => {
				error!("[{}] Unable to lock clientlist for add_client: {}", 
					self.channel, e)
			}
		}
	}

	pub fn remove_client(&self, client:&Client) {
		debug!("Looking to remove client: {}", client);
		match self.clientlist.lock() {
			Ok(mut list) => {				
				let mut x: Option<usize> = None;
				for (i,c) in list.iter().enumerate() {
					if Arc::as_ptr(c) == client {
						x = Some(i);
					}
				}
				if let Some(x) = x {
					debug!("Removing {} from clientlist", client);
					list.remove(x);
				}
			},
			Err(_) => ()
		}
	}
	pub fn core (&mut self) {
		loop {
			if *self.shutdown.read().unwrap() != 0 {
				warn!("Shutting down main loop");
				break;
			}
			let mut done_something = false;
			
			match self.rx.lock().unwrap().try_recv() {
				Ok(recv_msg) => {
					debug!("* {}", recv_msg);
					
					self.send_to_all(recv_msg);
					
					done_something = true;
				},
				Err(mpsc::TryRecvError::Empty) => (),
				Err(mpsc::TryRecvError::Disconnected) => warn!("central recv disconnected - all the clients gone?"),
			}
			
			if !done_something {
				std::thread::sleep(Duration::from_millis(500));
				// ping the writers to make sure they're still alive
				self.send_to_all(Message::Ping(Vec::new()));
			} else {
				debug!("* not sleeping");
			}
		}
	}
	fn send_to_all(&self, msg:Message) {
		if let Message::Text(msg) = msg.clone() {
			info!("* Sending msg '{:?}' to {} channels", msg, self.stats.num_clients());
		}
		
		let mut channels_to_be_removed = Vec::new();
		
		match self.central_outgoing.lock() {
			Ok(co) => {
				for (i,tx) in co.iter().enumerate() {
					match tx.send(msg.clone()) {
						Ok(_) => (),
						Err(_) => {
							// this channel is no longer good
							channels_to_be_removed.push(i);
						},
					}
				}
			}
			Err(e) => {
				error!("Cannot lock central outgoing: {}", e);
			}
		}
		if ! channels_to_be_removed.is_empty() {
			debug!("Going to remove {} channels from central_outgoing", channels_to_be_removed.len());
			if let Ok(mut co_write) = self.central_outgoing.lock() {
				loop {
					match channels_to_be_removed.pop() {
						Some(x) => {
							debug!("Dropping tx channel");
							co_write.remove(x);
						},
						None => break, // from loop
					};
				}
				self.stats.set_num_clients(co_write.len());
			}
		}
	}
}
