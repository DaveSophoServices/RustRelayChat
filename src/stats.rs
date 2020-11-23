use std::sync::RwLock;
use log::debug;

struct InternalStats {
    num_clients: usize,
    // ver is an continually incrementing unsigned 32. Each time our
    // struct is updated, so is ver. It doesn't matter if ver wraps,
    // as the calling code should check for !=, rather than > or <
    ver: u32,
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
		    ver: 0,
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
	    Ok(mut s) => {
		debug!("Updating stats version to: {}", s.ver+1);
		s.ver += 1;
		s.num_clients = num;
	    },
	    Err(_) => (),
	}
    }
    pub fn ver(&self) -> u32 {
	match self.int.read() {
	    Ok(s) => s.ver,
	    Err(_) => 0,
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
