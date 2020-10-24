use std::sync::RwLock;

struct InternalStats {
    num_clients: usize,
}

pub struct Stats {
    int: RwLock<InternalStats>,
}

impl Stats {
    pub fn new() -> Stats {
	Stats {
	    int: RwLock::new(
		InternalStats {
		    num_clients: 0,
		}),
	}
    }
    pub fn num_clients(&self) -> usize {
	match self.int.read() {
	    Ok(s) => s.num_clients,
	    Err(_) => 0,
	}
    }
    pub fn set_num_clients(&self, num:usize) {
	match self.int.write() {
	    Ok(mut s) => s.num_clients = num,
	    Err(_) => (),
	}
    }
    pub fn stat_msg(&self) -> tungstenite::Message {
	match self.int.read() {
	    Ok(s) => {
		tungstenite::Message::Text(
		    format!(r#"!*STAT {{"users":{}}}"#, s.num_clients)
		)
	    },
	    Err(_) => {
		tungstenite::Message::Text(
		    r#"!*STAT {{}}"#.to_string()
		)
	    },
	}
    }


}
