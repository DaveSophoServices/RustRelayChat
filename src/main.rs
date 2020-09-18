use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::server::accept;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();

    // this channel will be used by clients to put their messages when
    // they receive them from the user
    let (tx,rx) = mpsc::channel();
    let outgoing_list: Arc<Vec<mpsc::Sender<&str>>> = Arc::new(Vec::new());

    let central_outgoing = outgoing_list.clone();
    spawn (move || {
	loop {
	    let msg = rx.recv().unwrap();
	    println!("{}", msg);
	    for tx in *central_outgoing {
		tx.send(msg).unwrap();
	    }
	}
    });
    
    for stream in server.incoming() {
	let tx_clone = tx.clone();
	let (tx2, rx2) = mpsc::channel();
	&outgoing_list.push(tx2);
	    
	spawn (move || {
	    let mut websocket = accept(stream.unwrap()).unwrap();
	    websocket.get_ref().set_read_timeout(Some(Duration::new(0,100)));
	    
	    loop {
		let msg_res = websocket.read_message();
		match msg_res {
		    Ok(msg) => tx_clone.send(&msg.into_text().unwrap()).unwrap(),
		    Err(x) => println!("websocket error: {}", x),
		}
		// check rx2 for messages too
		let recv_res = rx2.try_recv();
		match recv_res {
		    Ok(recv_msg) => websocket.write_message(tungstenite::Message::text(recv_msg)).unwrap(),
		    Err(mpsc::TryRecvError::Empty) => println!("rx2 empty"),
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
