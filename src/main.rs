use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;
use tungstenite::error::Error;
use tungstenite::protocol::Message;
use std::sync::{Arc,RwLock,mpsc};
use std::time::Duration;

mod server;

fn main() {
    run(0);
}

fn run(timer: u64) {
    let shutdown = Arc::new(RwLock::new(0));

    let server = 
	match TcpListener::bind("127.0.0.1:9001") {
	    Ok(x) => x,
	    Err(x) => panic!("Cannot listen on port 9001: {}", x),
	};
    
    // this channel will be used by clients to put their messages when
    // they receive them from the user
    let (tx,rx) = mpsc::channel();
    let (newch_tx, newch_rx) = mpsc::channel();
    //    let outgoing_list: Arc<Vec<mpsc::Sender<&str>>> = Arc::new(Vec::new());

    let c_shutdown = shutdown.clone();
    // let central_outgoing = outgoing_list.clone();
    spawn (move || server::core(rx, newch_rx, c_shutdown));

    if timer > 0 {
	// this will shutdown the server in 10 seconds from starting
	let c_shutdown = shutdown.clone();
	spawn (move || {
	    std::thread::sleep(Duration::new(timer,0));
	    let mut n = c_shutdown.write().unwrap();
	    *n = 1;
	});
    }

    let channel_read_duration = Duration::from_secs(1);
    
    for stream in server.incoming() {
	let shutdown = shutdown.clone();

	let stream_unwrapped = stream.unwrap();

	let tx_clone = tx.clone();
	let (tx2, rx2) = mpsc::channel();
	newch_tx.send(tx2).unwrap();
	spawn (move || {
	    let addr = stream_unwrapped.peer_addr().unwrap();
	    dbg!(format!("new connection: {}", addr));
	    
	    let stream_clone = stream_unwrapped.try_clone();
	    let mut websocket = accept(stream_unwrapped).unwrap();
	    //websocket.get_ref().set_read_timeout(Some(Duration::new(0,100))).unwrap();
	    let mut websocket_recv =
		tungstenite::protocol::WebSocket::from_raw_socket(
		    stream_clone.unwrap(),
		    tungstenite::protocol::Role::Server,
		    None
		);

	    let pair_shutdown = Arc::new(RwLock::new(0));
	    let c_pair_shutdown = pair_shutdown.clone();
	    
	    let c_shutdown = shutdown.clone();
	    let c_addr = addr.clone();
	    spawn (move || {
		loop {
		    // check rx2 for messages too
		    if *c_shutdown.read().unwrap() != 0 {
			break;
		    }
		    if *c_pair_shutdown.read().unwrap() != 0 {
			break;
		    }
		    
		    let recv_res = rx2.recv_timeout(channel_read_duration);
		    match recv_res {
			// send it to the central thread
			Ok(recv_msg) => {
			    //dbg!(format!("[{}] Writing to websocket", c_addr));
			    match websocket.write_message(recv_msg) {
				Err(Error::ConnectionClosed) => break,
				Err(x) => {
				    // we got a fatal error from the connection
				    // it's probably died
				    break
				},
				Ok(x) => (),
			    }
			},
			Err(mpsc::RecvTimeoutError::Timeout) => (),
			Err(mpsc::RecvTimeoutError::Disconnected) => {
			    println!("rx2 disconnect");
			    break;
			},
		    }
		}
		println!("[{}] closed write loop", c_addr);
	    });
		   
	    loop {
		if *shutdown.read().unwrap() != 0 {
		    break;
		}
		if *pair_shutdown.read().unwrap() != 0 {
		    break;
		}
		
		let msg_res = websocket_recv.read_message();
		match msg_res {
		    Ok(msg) => {
			dbg!(format!("[{}] Sending msg to channel", addr));
			// has the message been handled as a command
			let mut handled = false;
			// handle the message if it's a command
			match msg.to_text() {
			    Ok(x) => {
				if x.starts_with('/') {
				    dbg!(x);
				    match x {
					"/QUIT" => {
					    // we're going to close out
					    println!("[{}] Going to close connection", addr);
					    websocket_recv.write_message(Message::Text("** Going to close connection".to_string()));
					    websocket_recv.write_message(Message::Close(None));
					    // signal the pair thread to shutdown
					    let mut ps = pair_shutdown.write().unwrap();
					    *ps = 1;
					}
					_ => {
					    println!("[{}] unknown command: {}", addr, x);
					}
				    }
				    // assume we've handled it
				    handled = true;
				}
			    }
			    Err(_) => { println!("[{}] Couldn't convert message to str", addr);
			    }
			}
			// don't print handled commands to central command.
			if !handled {
			    match tx_clone.send(msg) {
				Ok(_) => (),
				Err(x) => {
				    println!("ERR: unable to send msg to central: {}", x);
				},
			    }
			}
		    },
		    Err(Error::ConnectionClosed) => {
			println!("[{}] websocket closed", addr);
			break; // from loop
		    },
		    Err(Error::AlreadyClosed) => {
			println!("[{}] websocket already closed", addr);
			break; // from loop
		    },
		    Err(x) => {
			println!("[{}] websocket error: ({}) {}", addr, type_of(&x), x);
			break; // from loop
		    },
		}

		
		// if msg.is_binary() || msg.is_text() {
		//     
		// }
	    }
	    println!("[{}] closed Read loop", addr);
	});
    }
}

fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::sync::{Arc,Mutex};
    use std::thread::spawn;
    use tungstenite::client::{client,connect};
    use tungstenite::Message;
    
    #[test]
    fn it_starts() {
	// check it finishes
	let done = Arc::new(Mutex::new(0));
	let c_done = done.clone();
	let t = spawn(move||{ run(1); *c_done.lock().unwrap() = 1; });

	std::thread::sleep(Duration::new(3,0));
	assert_eq!(*done.lock().unwrap(), 1, "server finished in the alloted time");
    }

    #[test]
    fn it_works() {
	// start server for 10 seconds
	spawn(|| { run(10) });
	std::thread::sleep(Duration::new(1,0));
			   
	// connect to server with a websocket client
	let a = spawn( ||{
	    let mut ws = connect("ws://localhost:9001/").unwrap().0;
	    // wait for other thread to connect
	    std::thread::sleep(Duration::new(0,500));
	    // send a random string to first client
	    ws.write_message(Message::Text("going to send it".to_string())).unwrap();
	    ws.write_pending();
	});

	// connect again to server with a websocket client
	let b = spawn( ||{
	    let mut ws = connect("ws://localhost:9001/").unwrap().0;
	    // blocks until it gets a message
	    // it should be received on second client
	    let msg = ws.read_message().unwrap();
	    assert_eq!(msg.into_text().unwrap(), "going to send it".to_string());
	});

	a.join().unwrap();
	b.join().unwrap();
    }

    use std::collections::HashMap;
    
    #[test]
    fn load_test() {
	let max = 1000;
	// start the server for 30 seconds
	spawn ( || { run(60); } );
	// build a list of x number of random numbers

	// let the listener above get situated before we begin
	std::thread::sleep(Duration::new(1,500));
	
	let mut list = Vec::new();
	for x in 0..max {
	    list.push(format!("{}", x).to_string());
	}
	
	// clone it, then launch the listener client
	let mut hash = HashMap::new();
	for x in &list {
	    hash.insert(x.clone(), 1);
	}

	// save a ref to this hash for the end check
	//let hash_c = hash.clone();
	
	let listener = spawn ( move ||
		{
		    let mut ws = connect("ws://localhost:9001/").unwrap().0;
		    loop {
			let msg = ws.read_message().unwrap().into_text().unwrap();
			// we'll remove numbers from the list once we
			// 'hear' them in the channel
			if !hash.contains_key(&msg) {
			    panic!("Got key {} that wasn't in the map");
			} else {
			    hash.remove(&msg);
			}
			if hash.len() == 0 {
			    break;
			}
		    }
		});

	// let the listener above get situated before we begin
	std::thread::sleep(Duration::new(0,500));
	
	// launch x number of threads, passing in the random number they
	// will write to the server.
	let mut threads = Vec::new();
	for _ in 0..max {
	    let num = list.pop().unwrap();
	    threads.push(spawn ( move || {
		let mut ws = connect("ws://localhost:9001/").unwrap().0;
		ws.write_message(Message::Text(num)).unwrap();
		ws.write_pending();
		std::thread::sleep(Duration::new(50,0));
	    }));
	}
	    
	// wait for threads to finish
	let x = threads.len();
	for _ in 0..x {
	    let t = threads.pop().unwrap();
	    t.join().unwrap();
	}

	listener.join().unwrap();
	// check all the threads were heard by listener
//	assert_eq!(0, hash_c.len());
    }
    #[test]
    fn connect_write_close_test() {
	// start the server
	spawn ( || { run(10); } );

	std::thread::sleep(Duration::new(1,500));

	// one second timeout on reading
	let strm = std::net::TcpStream::connect("localhost:9001").unwrap();
	strm.set_read_timeout(Some(Duration::new(1,0)));

	let mut ws = client("ws://localhost:9001/", strm).unwrap().0;
	ws.write_message(Message::Text("/QUIT".to_string())).unwrap();
	ws.write_pending();

	loop {
	    match ws.read_message() {
		Ok(x) => { println!("returned message: '{}'", x.into_text().unwrap()); }
		Err(x) => {
		    println!("returned error: {}:{}", type_of(&x), x);
		    break;
		}
	    }
	}
	
	// try writing a new message
	match ws.write_message(Message::Text("Am I connected".to_string())) {
	    Ok(_) => { panic!("Websocket appears to still be connected"); },
	    Err(_) => () // that's ok :-)
	}
    }
	
}
