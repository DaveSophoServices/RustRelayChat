use std::net::TcpListener;
use std::thread::spawn;
use std::sync::{Arc};
use std::time::Duration;

mod server;
mod stats;
mod websocket_headers;
mod config;
mod client;
mod dblog;

use log::{info};

fn main() {
    init_log();
    run(0);
}

fn run(timer: u64) {
    // load config
    let config = Arc::new(load_config());
    let server = 
	match TcpListener::bind(format!("0.0.0.0:{}", config.port)) {
	    Ok(x) => x,
	    Err(x) => panic!("Cannot listen on port {}: {}", config.port, x),
	};    
    info!("Listening on port {}", config.port);
    // this channel will be used by clients to put their messages when
    // they receive them from the user
    let main_server = Arc::new(server::Server::new(config.clone()));
    
    //let (tx,rx) = mpsc::channel();
    //let (newch_tx, newch_rx) = mpsc::channel();
    //    let outgoing_list: Arc<Vec<mpsc::Sender<&str>>> = Arc::new(Vec::new());

    //let s = server::Server::new(rx, newch_rx, c_shutdown);
    //let stats = s.get_stats();
    // let central_outgoing = outgoing_list.clone();
    //s.spawn_core();

    if timer > 0 {
	// this will shutdown the server in 10 seconds from starting
	let c_shutdown = main_server.shutdown_ref();
	spawn (move || {
	    std::thread::sleep(Duration::new(timer,0));
	    let mut n = c_shutdown.write().unwrap();
	    *n = 1;
	});
    }

    
    for stream in server.incoming() {
	if let Ok(stream) = stream {
	    client::new(stream, main_server.clone());
	}
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
    use tungstenite::{Message,protocol::WebSocket};
    use std::net::TcpStream;

    fn ws_with_timeout() -> WebSocket<TcpStream> {
	return ws_with_timeout_room("");
    }
    
    fn ws_with_timeout_room(room: &str) -> WebSocket<TcpStream> {
	let strm = TcpStream::connect("localhost:9001").unwrap();
	strm.set_read_timeout(Some(Duration::new(1,0)));

	return client(format!("ws://localhost:9001/{}", room), strm).unwrap().0;
    }
    
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
	let max = 100;
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
		std::thread::sleep(Duration::new(5,0));
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
	let mut ws = ws_with_timeout();
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

    #[test]
    fn stats_info() {
	spawn ( || { run(10); } );
	std::thread::sleep(Duration::new(1,500));
	let mut ws = ws_with_timeout();

	// connect to the server and it should tell us how many people are connected
	let msg = ws.read_message().unwrap();
	assert_eq!(msg.into_text().unwrap(), r#"!*STAT {"users":1}"#);
	ws.write_message(Message::Text("/QUIT".to_string())).unwrap();
	std::thread::sleep(Duration::new(1,0));
    }

    #[test]
    fn diff_chat_rooms() {
	spawn ( || { run (10); } );
	std::thread::sleep(Duration::new(1,500));

	let mut ws = ws_with_timeout_room("one");
	let mut ws2 = ws_with_timeout_room("one");
	let mut ws3 = ws_with_timeout_room("two");

	// get the STAT messages out of the way
	let mut msg = ws.read_message().unwrap();
	msg = ws2.read_message().unwrap();
	msg = ws3.read_message().unwrap();

	ws2.write_message(Message::Text("abc".to_string())).unwrap();
	msg = ws.read_message().unwrap();
	assert_eq!(msg.into_text().unwrap(), "abc");
	match ws3.read_message() {
	    Ok (_) => panic!("not supposed to receive a response on ws3"),
	    Err (_) => (), // Ok, as we got a timeout
	}
	
    }
	
}

fn init_log() {
    simplelog::CombinedLogger::init(
	vec![
	    simplelog::TermLogger::new(
		simplelog::LevelFilter::Debug,
		simplelog::Config::default(),
		simplelog::TerminalMode::Mixed),
	    simplelog::WriteLogger::new(
		simplelog::LevelFilter::Info,
		simplelog::Config::default(),
		std::fs::File::create("chat.log").unwrap()),
	]).unwrap();
}

fn load_config() -> config::Config {
    // read the text file
    if let Ok(s) = std::fs::read_to_string("chat.json") {
	// json deserialize it
	config::parse_json(&s)
    } else {
	config::default()
    }
}
