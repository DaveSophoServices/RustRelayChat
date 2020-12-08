
pub mod logmessage;

use crate::config;

use std::thread::spawn;
use logmessage::LogMessage;
use std::sync::{Arc,Mutex,mpsc};
use log::{error};
use mysql::*;
use mysql::prelude::*;

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
    let url = format!("mysql://{}:{}@{}:{}/{}",
		      config.dbuser,
		      config.dbpass,
		      config.dbhost,
		      config.dbport,
		      config.dbname);

    let pool = match Pool::new(url.clone()) {
	Ok(p) => p,
	Err(e) => {
	    error!("Cannot create a mysql pool with url {}: {}", url, e);
	    return;
	}
    };
	
    loop {
	let mut conn = match pool.get_conn() {
	    Ok(c) => c,
	    Err(e) => { err_wait(e); continue; }
	};
	// ok, we have a conn
	
    }
    // check table exists
}

// prints an error and waits a short time
fn err_wait(m: impl std::fmt::Display) {
    error!("Unable to get connection: {}", m);
    std::thread::sleep(std::time::Duration::from_secs(60));
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
