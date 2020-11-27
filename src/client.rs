use std::net::{SocketAddr,TcpStream};
use std::sync::{Arc,Mutex,RwLock,mpsc};
use tungstenite::{accept_hdr,Error,Message,WebSocket};
use std::thread::spawn;
use std::time::Duration;

use crate::server::channel_server::ChannelServer;
use crate::stats::Stats;
use crate::server::Server;
use crate::websocket_headers;

use log::{debug,info,warn,error};

pub struct Client {
    name: String,
    addr: SocketAddr,
    websocket: Arc<WebSocket<TcpStream>>,
    ch: Arc<ChannelServer>,
    pair_shutdown: Arc<RwLock<u32>>,
    shutdown: Arc<RwLock<u32>>,
    stats: Arc<Stats>,
    tx: Arc<Mutex<mpsc::Sender<Message>>>,
    rx: Arc<Mutex<mpsc::Receiver<Message>>>,
    main_server: Arc<Server>,
}

pub fn new(stream: TcpStream, main_server: Arc<Server>) -> Option<Arc<Client>> {
    let ws_hdr_cb = websocket_headers::new_callback();
    let ws_hdr = ws_hdr_cb.hdr();
    let addr = stream.peer_addr().unwrap();
    let websocket = Arc::new(accept_hdr(
	stream, ws_hdr_cb
    ).unwrap());
    
    let ch = match main_server.get(ws_hdr.clone()) {
	Some(x) => Arc::new(x),
	None => {
	    warn!("[{}] tried to create channel {:?} but not allowed", addr, ws_hdr);
	    return None;
	},
    };

    let (tx,rx) = ch.get_tx_rx();

    /*
	    let stream_clone = stream_unwrapped.try_clone();
	    let mut websocket_recv =
		tungstenite::protocol::WebSocket::from_raw_socket(
		    stream_clone.unwrap(),
		    tungstenite::protocol::Role::Server,
		    None
		);
     */
    let stats = ch.get_stats();
    let r = Arc::new(Client {
	name: "user".to_string(),
	addr,
	websocket,
	ch,
	rx: Arc::new(Mutex::new(rx)),
	tx: Arc::new(Mutex::new(tx)),
	pair_shutdown: Arc::new(RwLock::new(0)),
	shutdown: main_server.shutdown_ref(),
	stats,
	main_server,
    });
    info!("new connection: {}", r.addr);

    // spin off the threads to do the receiving and sending
    sender(r.clone());
    receiver(r.clone());

    return Some(r);	
}

// central -> webbrowser socket
fn sender(client: Arc<Client>) {
    let channel_read_duration = Duration::from_secs(1);

    spawn(move || {
	// WRITE Loop
	let mut old_stats_version: u32 = 0xFFFFFFFF;
	loop {
	    if client.check_shutdowns() != 0 {
		debug!("[{}] write loop shutdown requested", client.addr);
		break;
	    }
	    if client.stats.ver() != old_stats_version {
		old_stats_version = client.stats.ver();
		client.write(client.stats.stat_msg());
	    }

	    // check if anything from central
	    if let Ok(rx) = client.rx.lock() {
		match rx.recv_timeout(channel_read_duration) {
		    Ok(msg) => {
			if let Message::Ping(_) = msg {
			    // ignore it. Just central checking we're alive
			} else {
			    client.write(msg);
			}
		    },
		    Err(mpsc::RecvTimeoutError::Timeout) => (), // ignore it
		    Err(mpsc::RecvTimeoutError::Disconnected) => 
			client.mark_connection_closed(),
		}
	    }	
	}
	debug!("[{}] closed write loop", client.addr);
    });
}

// webbrowser socket -> central
fn receiver(client: Arc<Client>) {
    loop {
	if client.check_shutdowns() != 0 {
	    debug!("[{}] closing read loop due to client shutdown req",
		   client.addr);
	    break;
	}
	match client.websocket.read_message() {
	    Ok(Message::Text(msg)) => {
		debug!("[{}] Sending msg ({:?}) to central", client.addr, msg);
		let mut handled = false;
		if msg.starts_with('/') {
		    debug!("[{}] {} command", client.addr, msg);
		    match msg.as_str() {
			"/QUIT" => {
			    debug!("[{}] Going to close connection",
				   client.addr);
			    client.close("** Going to close connection.");
			},
			_ => {
			    warn!("[{}] unknown command: {}", client.addr, msg);
			}
		    }
		    handled = true;
		}

		if !handled {
		    client.to_central(msg);
		}
	    }
	    Ok(_) => (), // ignore other websocket message types
	    Err(Error::ConnectionClosed) => {
		info!("[{}] websocket closed.", client.addr);
		client.mark_connection_closed();
	    },
	    Err(Error::AlreadyClosed) => {
		info!("[{}] websocket already closed.", client.addr);
		client.mark_connection_closed();
	    },	    
	    Err(e) => {
		info!("[{}] websocket error: ({}) {}",
		      client.addr, type_of(&e), e);
		client.mark_connection_closed();
	    },
	}
    } // end of loop
    debug!("[{}] closed read loop.", client.addr);
}

fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}


impl Client {
    fn check_shutdowns(&self) -> u32 {
	if let Ok(i) = self.shutdown.read() {
	    if *i != 0 {
		return *i;
	    }
	}
	if let Ok(i) = self.pair_shutdown.read() {
	    if *i != 0 {
		return *i;
	    }
	}
	return 0;
    }
    
    fn close(&self, msg: &str) {
	self.write(Message::Text(msg.to_string()));
	self.write(Message::Close(None));
	self.mark_connection_closed();
    }
    
    // called when we have an error that wants us to terminate
    fn mark_connection_closed(&self) {
	if let Ok(mut i) = self.pair_shutdown.write() {
	    debug!("Marking our connection pair as closing.");
	    *i = 1;
	}
    }    

    fn to_central(&self, msg: String) {
	if let Ok(tx) = self.tx.lock() {
	    if let Err(e) = tx.send(Message::Text(msg)) {
		error!("[{}] unable to send msg to central: {}",
		       self.addr, e);
	    }
	}
	    
    }
    
    fn write(&self, msg: Message) {
	match self.websocket.write_message(msg) {
	    Err(Error::ConnectionClosed) => self.mark_connection_closed(),
	    Err(e) => {
		// we got a fatal error from the connection
		// it's probably died
		debug!("[{}] shutdown due to websocket error: {}",
		       self.addr, e);
		self.mark_connection_closed();
	    },
	    Ok(_) => (),
	}
    }
}
