
pub mod logmessage;

use logmessage::LogMessage;
use std::sync::{Arc,Mutex,mpsc};
use log::{error};

pub struct DBLog {
    rx: Arc<Mutex<mpsc::Receiver<LogMessage>>>,
    tx: Arc<Mutex<mpsc::Sender<LogMessage>>>,
}

pub fn new() -> DBLog {
    let (tx,rx) = mpsc::channel();
    DBLog {
	rx:Arc::new(Mutex::new(rx)),
	tx:Arc::new(Mutex::new(tx)),
    }
}

impl DBLog {
    pub fn get_sender(&self) -> Option<mpsc::Sender<LogMessage>> {
	match self.tx.lock() {
	    Ok(tx) => Some(tx.clone()),
	    Err(e) => {
		error!("Unable to lock DB tx master");
		None
	    },
	}
    }	
}
