use std::sync::mpsc;
use std::time::Duration;


pub fn core (
    rx: mpsc::Receiver<tungstenite::Message>,
    newch_rx: mpsc::Receiver<mpsc::Sender<tungstenite::Message>>,
    shutdown:std::sync::Arc<std::sync::RwLock<i32>>
) {
    let mut central_outgoing: Vec<mpsc::Sender<tungstenite::Message>> = Vec::new();
    loop {
	if *shutdown.read().unwrap() != 0 {
	    break;
	}
	match rx.try_recv() {
	    Ok(recv_msg) => {
		println!("{}", recv_msg);
		for tx in &central_outgoing {
		    tx.send(recv_msg.clone()).unwrap();
		}
	    },
	    Err(mpsc::TryRecvError::Empty) => (),
	    Err(mpsc::TryRecvError::Disconnected) => println!("central recv disconnected"),
	}

	// any new transmit clients
	match newch_rx.try_recv() {
	    Ok(new_channel) => central_outgoing.push(new_channel),
	    Err(mpsc::TryRecvError::Empty) => (),
	    Err(mpsc::TryRecvError::Disconnected) => println!("new channel recv disconnected"),
	}

	std::thread::sleep(Duration::new(1,0));
    }
    
}

pub fn sendrecv() {
}
