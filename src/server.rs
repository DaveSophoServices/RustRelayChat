use std::sync::{mpsc,Arc,RwLock};
use std::time::Duration;
use tungstenite::Message;
use std::thread::spawn;

use crate::stats::Stats;

pub struct Server {
    central_outgoing: Vec<mpsc::Sender<Message>>,
    rx: mpsc::Receiver<Message>,
    newch_rx: mpsc::Receiver<mpsc::Sender<Message>>,
    shutdown: Arc<RwLock<i32>>,
    stats: Arc<Stats>,
}


// num_clients will return the number of active clients
// should be based on the number of connections, or tx channels
impl Server {
    pub fn new(rx: mpsc::Receiver<tungstenite::Message>,
	   newch_rx: mpsc::Receiver<mpsc::Sender<tungstenite::Message>>,
	   shutdown:std::sync::Arc<std::sync::RwLock<i32>>
    ) -> Server {
	Server {
	    central_outgoing: Vec::new(),
	    rx,
	    newch_rx,
	    shutdown,
	    stats: Arc::new(Stats::new()),
	}
    }
    pub fn get_stats(&self) -> Arc<Stats> {
	return self.stats.clone();
    }
    pub fn spawn_core(mut self) {
    	spawn( move || self.core() );
    }
    pub fn core (&mut self) {
	// we may want to remove
	let mut channels_to_be_removed = Vec::new();

	loop {
	    if *self.shutdown.read().unwrap() != 0 {
		println!("Shutting down main loop");
		break;
	    }
	    let mut done_something = false;
	    
	    match self.rx.try_recv() {
		Ok(recv_msg) => {
		    println!("* {}", recv_msg);

		    println!("* Sending msg '{}' to {} channels", recv_msg, self.stats.num_clients());
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
				    dbg!("Dropping tx channel");
				    self.central_outgoing.remove(x);
				},
				None => break, // from loop
			    };
			}
		    }
		    done_something = true;
		},
		Err(mpsc::TryRecvError::Empty) => (),
		Err(mpsc::TryRecvError::Disconnected) => println!("central recv disconnected - all the clients gone?"),
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
		Err(mpsc::TryRecvError::Disconnected) => println!("new channel recv disconnected"),
	    }

	    if !done_something {
		std::thread::sleep(Duration::new(0,500));
	    } else {
		println!("* not sleeping");
	    }
	}
    }	
}

pub fn sendrecv() {
}
