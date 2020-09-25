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
	let mut done_something = false;
	
	match rx.try_recv() {
	    Ok(recv_msg) => {
		println!("* {}", recv_msg);
		for tx in &central_outgoing {
		    println!("* Sending msg '{}'", recv_msg);
		    tx.send(recv_msg.clone()).unwrap();
		}
		done_something = true;
	    },
	    Err(mpsc::TryRecvError::Empty) => (),
	    Err(mpsc::TryRecvError::Disconnected) => println!("central recv disconnected"),
	}

	// any new transmit clients
	match newch_rx.try_recv() {
	    Ok(new_channel) => {
		central_outgoing.push(new_channel);
		println!("* received a send channel");
		done_something = true;
	    },
	    Err(mpsc::TryRecvError::Empty) => (),
	    Err(mpsc::TryRecvError::Disconnected) => println!("new channel recv disconnected"),
	}

	if (!done_something) {
	    std::thread::sleep(Duration::new(0,500));
	} else {
	    println!("* not sleeping");
	}
    }
    
}

pub fn sendrecv() {
}
