use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;
use std::sync::{Arc,RwLock,mpsc};
use std::time::Duration;

mod server;

fn main() {
    run(0);
}

fn run(timer: u64) {
    let shutdown = Arc::new(RwLock::new(0));
    
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

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
	   
    for stream in server.incoming() {
	let tx_clone = tx.clone();
	let (tx2, rx2) = mpsc::channel();
	let shutdown = shutdown.clone();
	
	newch_tx.send(tx2).unwrap();
	spawn (move || {
	    let stream_unwrapped = stream.unwrap();
	    let stream_clone = stream_unwrapped.try_clone();
	    let mut websocket = accept(stream_unwrapped).unwrap();
	    //websocket.get_ref().set_read_timeout(Some(Duration::new(0,100))).unwrap();
	    let mut websocket_recv =
		tungstenite::protocol::WebSocket::from_raw_socket(
		    stream_clone.unwrap(),
		    tungstenite::protocol::Role::Server,
		    None
		);

	    let c_shutdown = shutdown.clone();
	    spawn (move || {
		loop {
		    // check rx2 for messages too
		    if *c_shutdown.read().unwrap() != 0 {
			break;
		    }
		    
		    let recv_res = rx2.recv();
		    match recv_res {
			// send it to the central thread
			Ok(recv_msg) => websocket.write_message(recv_msg).unwrap(),
			//Err(mpsc::TryRecvError::Empty) => (),
			//Err(mpsc::TryRecvError::Disconnected) => println!("rx2 disconnect"),
			Err(x) => println!("rx2: {}", x),
		    }
		}
	    });
		   
	    loop {
		if *shutdown.read().unwrap() != 0 {
		    break;
		}
		let msg_res = websocket_recv.read_message();
		match msg_res {
		    Ok(msg) => {
			tx_clone.send(msg).unwrap()
		    },
		    // Err(tungstenite::error::Error::Io(x)) => {
		    // 	if let Some(raw_error) = x.raw_os_error() {
		    // 	    if raw_error == 11 {
		    // 	    } else {
		    // 		println!("websocket error: ({}) {}", type_of(&x), x);
		    // 	    }
		    // 	}
		    // },
		    Err(tungstenite::error::Error::ConnectionClosed) => {
			println!("websocket closed");
			break; // from loop
		    },
		    Err(x) => {
			println!("websocket error: ({}) {}", type_of(&x), x)
		    },
		}


		
		// if msg.is_binary() || msg.is_text() {
		//     
		// }
	    }
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
    use tungstenite::client::connect;
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
	// start server 
	spawn(|| { run(0) });
	std::thread::sleep(Duration::new(0,500));
			   
	// connect to server with a websocket client
	let a = spawn( ||{
	    let mut ws = connect("ws://localhost:9001/").unwrap().0;
	    // wait for other thread to connect
	    std::thread::sleep(Duration::new(0,500));
	    // send a random string to first client
	    ws.write_message(Message::Text("going to send it".to_string())).unwrap();
	    
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
}
