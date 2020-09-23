use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;
use std::sync::mpsc;

mod server;

fn main() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    // this channel will be used by clients to put their messages when
    // they receive them from the user
    let (tx,rx) = mpsc::channel();
    let (newch_tx, newch_rx) = mpsc::channel();
    //    let outgoing_list: Arc<Vec<mpsc::Sender<&str>>> = Arc::new(Vec::new());

    // let central_outgoing = outgoing_list.clone();
    spawn (move || server::core(rx, newch_rx));
    
    for stream in server.incoming() {
	let tx_clone = tx.clone();
	let (tx2, rx2) = mpsc::channel();
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

	    spawn (move || {
		loop {
		    // check rx2 for messages too
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
