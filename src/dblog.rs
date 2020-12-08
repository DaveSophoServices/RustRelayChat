
pub mod logmessage;

use crate::config;

use std::thread::spawn;
use logmessage::LogMessage;
use std::sync::{Arc,Mutex,mpsc};
use log::{debug,error};
use mysql::*;
use mysql::prelude::*;

#[derive(Clone)]
pub struct DBLog {
    tx: Arc<Mutex<mpsc::Sender<LogMessage>>>,
    ch_lock: Arc<Mutex<i32>>,
}

pub fn new(config:Arc<config::Config>) -> DBLog {
    let (tx,rx) = mpsc::channel();
    // spin off the rx into a thread to await database messages
    let dbl = DBLog {
	tx:Arc::new(Mutex::new(tx)),
	ch_lock:Arc::new(Mutex::new(0)),
    };

    let dbc = dbl.clone();
    spawn( || logger(rx, config, dbc));
    return dbl;
}

fn logger(rx: mpsc::Receiver<LogMessage>, config:Arc<config::Config>, dbl:DBLog) {
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
	    Err(e) => { err_wait("Unable to get conn",e); continue; }
	};
	// ok, we have a conn
	loop {
	    // wait for a message
	    if let Ok(_) = dbl.ch_lock.lock() {
		match rx.recv_timeout(std::time::Duration::from_secs(1)) {
		    Ok(m) => {
			// this is a LogMessage
			debug!("About to log our message to the DB");
			match conn.exec_drop("INSERT INTO chat_log (username,address,channel,stamp,message)
                              VALUES (:username,:address,:channel,FROM_UNIXTIME(:stamp),:message)",
					     params! {
						 "username" => m.user,
						 "address" => format!("{}",m.addr),
						 "channel" => m.channel,
						 "stamp" => m.datetime.timestamp(),
						 "message" => m.message,
					     }
			) {
			    Ok(_) => (),
			    Err(e) => error!("Failed to write to database: {}", e),
			}		    
		    }
		    Err(mpsc::RecvTimeoutError::Timeout) => (),
		    Err(e) => { err_wait("recv error", e); break; /* from inner loop. we will attempt to pickup a new connection and try again */ }
		}
	    }
	    std::thread::sleep(std::time::Duration::from_millis(200));
	}
	
    }
    // check table exists
}

// prints an error and waits a short time
fn err_wait(l:&str, m: impl std::fmt::Display) {
    error!("{}: {}", l, m);
    std::thread::sleep(std::time::Duration::from_secs(60));
}
    
impl DBLog {
    pub fn get_sender(&self) -> Option<mpsc::Sender<LogMessage>> {
	debug!("Cloning a tx for the db logger");
	if let Ok(_) = self.ch_lock.lock() {
	    debug!("About to return the logger");
	    match self.tx.lock() {
		Ok(tx) => Some(tx.clone()),
		Err(e) => {
		    error!("Unable to lock DB tx master: {}", e);
		    None
		},
	    }
	} else {
	    None
	}
    }	
}
