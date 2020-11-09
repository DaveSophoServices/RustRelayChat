use std::sync::{mpsc,Arc,RwLock};
use std::time::Duration;

use std::thread::spawn;
use log::debug;
use tungstenite::Message;
use crate::stats::Stats;

pub struct ChannelServer {
    central_outgoing: Vec<mpsc::Sender<Message>>,
    tx: mpsc::Sender<Message>,
    rx: mpsc::Receiver<Message>,
    newch_rx: mpsc::Receiver<mpsc::Sender<Message>>,
    shutdown: Arc<RwLock<i32>>,
    stats: Arc<Stats>,
    channel: String,
}

impl ChannelServer {
    pub fn new(	   newch_rx: mpsc::Receiver<mpsc::Sender<tungstenite::Message>>,
	       shutdown:std::sync::Arc<std::sync::RwLock<i32>>,
	       channel:String,
    ) -> ChannelServer {
	let (tx,rx) = mpsc::channel();
	ChannelServer {
	    central_outgoing: Vec::new(),
	    tx,
	    rx,
	    newch_rx,
	    shutdown,
	    stats: Arc::new(Stats::new()),
	    channel,
	}
    }
    pub fn get_stats(&self) -> Arc<Stats> {
	return self.stats.clone();
    }
    pub fn spawn_core(mut self) {
    	spawn( move || self.core() );
    }
    pub fn get_tx_rx(&self) -> (mpsc::Sender<tungstenite::Message>,
				mpsc::Receiver<tungstenite::Message>)
    {
	// this pair lets the websocket client receive from us. We return the rx to them, keeping tx in our list
	let (tx,rx) = mpsc::channel();
	self.central_outgoing.push(tx);

	// clone the transmit end of our receiver so that clients can send to us
	(self.tx.clone(), rx)
    }
    pub fn core (&mut self) {
	// we may want to remove
	let mut channels_to_be_removed = Vec::new();

	loop {
	    if *self.shutdown.read().unwrap() != 0 {
		warn!("Shutting down main loop");
		break;
	    }
	    let mut done_something = false;
	    
	    match self.rx.try_recv() {
		Ok(recv_msg) => {
		    debug!("* {}", recv_msg);

		    info!("* Sending msg '{}' to {} channels", recv_msg, self.stats.num_clients());
		    for (i,tx) in self.central_outgoing.iter().enumerate() {
			match tx.send(recv_msg.clone()) {
			    Ok(_) => (),
			    Err(_) => {
				// this channel is no longer good
				channels_to_be_removed.push(i);
			    },
			}
		    }
		    if ! channels_to_be_removed.is_empty() {
			loop {
			    match channels_to_be_removed.pop() {
				Some(x) => {
				    debug!("Dropping tx channel");
				    self.central_outgoing.remove(x);
				},
				None => break, // from loop
			    };
			}
		    }
		    done_something = true;
		},
		Err(mpsc::TryRecvError::Empty) => (),
		Err(mpsc::TryRecvError::Disconnected) => warn!("central recv disconnected - all the clients gone?"),
	    }

	    // any new transmit clients
	    match self.newch_rx.try_recv() {
		Ok(new_channel) => {
		    self.central_outgoing.push(new_channel);
		    self.stats.set_num_clients(self.central_outgoing.len());
		    //println!("* received a send channel");
		    done_something = true;
		},
		Err(mpsc::TryRecvError::Empty) => (),
		Err(mpsc::TryRecvError::Disconnected) => debug!("new channel recv disconnected"),
	    }

	    if !done_something {
		std::thread::sleep(Duration::new(0,500));
	    } else {
		debug!("* not sleeping");
	    }
	}
    }	
}
