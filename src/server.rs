use std::sync::mpsc;
use std::time::Duration;


pub fn core (
    rx: mpsc::Receiver<tungstenite::Message>,
    newch_rx: mpsc::Receiver<mpsc::Sender<tungstenite::Message>>,
    shutdown:std::sync::Arc<std::sync::RwLock<i32>>
) {
    let mut central_outgoing: Vec<mpsc::Sender<tungstenite::Message>> = Vec::new();
    // we may want to remove
    let mut channels_to_be_removed = Vec::new();

    loop {
	if *shutdown.read().unwrap() != 0 {
	    println!("Shutting down main loop");
	    break;
	}
	let mut done_something = false;
	
	match rx.try_recv() {
	    Ok(recv_msg) => {
		println!("* {}", recv_msg);

		let mut i = 0;
		println!("* Sending msg '{}' to {} channels", recv_msg, central_outgoing.len());
		for tx in &central_outgoing {
		    match tx.send(recv_msg.clone()) {
			Ok(x) => (),
			Err(x) => {
			    // this channel is no longer good
			    channels_to_be_removed.push(i);
			},
		    }
		    i+=1;
		}
		if ! channels_to_be_removed.is_empty() {
		    loop {
			match channels_to_be_removed.pop() {
			    Some(x) => {
				dbg!("Dropping tx channel");
				central_outgoing.remove(x);
			    },
			    None => break, // from loop
			};
		    }
		}
		done_something = true;
	    },
	    Err(mpsc::TryRecvError::Empty) => (),
	    Err(mpsc::TryRecvError::Disconnected) => println!("central recv disconnected - all the clients gone?"),
	}

	// any new transmit clients
	match newch_rx.try_recv() {
	    Ok(new_channel) => {
		central_outgoing.push(new_channel);
		//println!("* received a send channel");
		done_something = true;
	    },
	    Err(mpsc::TryRecvError::Empty) => (),
	    Err(mpsc::TryRecvError::Disconnected) => println!("new channel recv disconnected"),
	}

	if !done_something {
	    std::thread::sleep(Duration::new(0,500));
	} else {
	    println!("* not sleeping");
	}
    }
    
}

pub fn sendrecv() {
}
