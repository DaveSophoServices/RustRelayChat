use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;
use std::sync::mpsc;
use std::time::Duration;

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
	    let mut websocket = accept(stream.unwrap()).unwrap();
	    websocket.get_ref().set_read_timeout(Some(Duration::new(0,100))).unwrap();
	    
	    loop {
		let msg_res = websocket.read_message();
		match msg_res {
		    Ok(msg) => {
//			let msg: &str = 
			tx_clone.send(msg).unwrap()
		    },
		    Err(x) => println!("websocket error: {}", x),
		    // TODO handle IO error: Resource temporarily unavailable - we're using non-blocking reads
		}
		// check rx2 for messages too
		let recv_res = rx2.try_recv();
		match recv_res {
		    Ok(recv_msg) => websocket.write_message(recv_msg).unwrap(),
		    Err(mpsc::TryRecvError::Empty) => (),
		    Err(mpsc::TryRecvError::Disconnected) => println!("rx2 disconnect"),
		}
		// send it to the central thread

		
		// if msg.is_binary() || msg.is_text() {
		//     
		// }
	    }
	});
    }
}
