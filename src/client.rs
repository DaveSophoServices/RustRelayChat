use std::net::{SocketAddr,TcpStream};
use std::sync::{Arc,RwLock,mpsc};
use tungstenite::{Message,WebSocket};

use crate::server::channel_server::ChannelServer;
use crate::stats::Stats;
use crate::server::Server;

pub struct Client {
    name: String,
    addr: SocketAddr,
    websocket: Arc<WebSocket>,
    ch: Arc<ChannelServer>,
    pair_shutdown: i32,
    shutdown: Arc<RwLock<i32>>,
    stats: Arc<Stats>,
    tx: Arc<mpsc::Sender<Message>>,
    rx: Arc<mpsc::Receiver<Message>>,
    main_server: Arc<Server>,
}

pub fn new(stream: TcpStream, main_server: Arc<Server>) -> Arc<Client> {
    let stream_unwrapped = stream.unwrap();
    let ws_hdr_cb = websocket_headers::new_callback();
    let ws_hdr = ws_hdr_cb.hdr();
    let mut websocket = accept_hdr(
	stream_unwrapped, ws_hdr_cb
    ).unwrap();
    
    let ch = match main_server.get(ws_hdr.clone()) {
	Some(x) => x,
	None => {
	    warn!("[{}] tried to create channel {:?} but not allowed", addr, ws_hdr);
	    return
	},
    };

    let (rx,tx) = ch.get_tx_rx();

    /*
	    let stream_clone = stream_unwrapped.try_clone();
	    let mut websocket_recv =
		tungstenite::protocol::WebSocket::from_raw_socket(
		    stream_clone.unwrap(),
		    tungstenite::protocol::Role::Server,
		    None
		);
     */
    let ref = Arc::new(Client {
	name: "user",
	addr: stream_unwrapped.peer_addr().unwrap(),
	websocket,
	ch,
	rx,
	tx,
	pair_shutdown: 0,
	shutdown: main_server.shutdown_ref(),
	stats: ch.get_stats(),
	main_server,
    });
    info!("new connection: {}", ref.addr);

    // spin off the threads to do the receiving and sending
    sender(ref.clone());
    receiver(ref.clone());

    return ref;	
}

fn sender(client: Arc<Client>) {
    spawn(move || {
	// WRITE Loop
	let mut old_stats_version: u32 = 0xFFFFFFFF;
	loop {
	    if client.check_shutdowns() != 0 {
		debug!("[{}] write loop shutdown requested", client.addr);
		break;
	    }
	}
    });
}
fn receiver(client: Arc<Client>) {
    
}


impl Client {
    fn check_shutdowns() -> i32 {
	if let Ok(i) = client.shutdown.read() {
	    if *i != 0 {
		return *i;
	    }
	}
	if let Ok(i) = client.pair_shutdown.read() {
	    if *i != 0 {
		return *i;
	    }
	}
	return 0;
    }
}
