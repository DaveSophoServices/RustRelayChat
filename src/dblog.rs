
pub mod logmessage;

use crate::config;

use std::thread::spawn;
use logmessage::LogMessage;
use std::sync::{Arc,Mutex,mpsc};
use log::{error};


pub struct DBLog {
    tx: Arc<Mutex<mpsc::Sender<LogMessage>>>,
}

pub fn new(config:Arc<config::Config>) -> DBLog {
    let (tx,rx) = mpsc::channel();
    // spin off the rx into a thread to await database messages
    spawn( || logger(rx, config));
    DBLog {
	tx:Arc::new(Mutex::new(tx)),
    }
}

fn logger(rx: mpsc::Receiver<LogMessage>, config:Arc<config::Config>) {

}

impl DBLog {
    pub fn get_sender(&self) -> Option<mpsc::Sender<LogMessage>> {
	match self.tx.lock() {
	    Ok(tx) => Some(tx.clone()),
	    Err(e) => {
		error!("Unable to lock DB tx master: {}", e);
		None
	    },
	}
    }	
}
