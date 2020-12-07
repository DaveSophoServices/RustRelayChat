
pub mod logmessage;

use logmessage::LogMessage;
use std::sync::{Arc,Mutex,mpsc};

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
