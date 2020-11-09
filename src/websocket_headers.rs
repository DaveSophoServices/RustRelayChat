use tungstenite::handshake::server::{Callback,ErrorResponse,Request,Response};
use std::sync::{Arc,RwLock};
use log::{debug,error};
#[derive(Debug)]
pub struct WebsocketHeaders {
    pub uri: Option<http::uri::Uri>,
}

pub struct WebsocketHeadersCB {
    data: Arc<RwLock<WebsocketHeaders>>,
}

impl Callback for WebsocketHeadersCB {
    fn on_request(self, request: &Request, response: Response)
		  -> Result<Response, ErrorResponse> {
	self.set_uri(request.uri().clone());
	debug!("URI: {}", request.uri());
	Ok(response)
    }
}

pub fn new_callback() -> WebsocketHeadersCB {
    WebsocketHeadersCB {
	data: Arc::new(RwLock::new(
	    WebsocketHeaders {
		uri: None,
	    }))
    }
}

impl WebsocketHeadersCB {
    pub fn hdr(&self) -> Arc<RwLock<WebsocketHeaders>> {
	self.data.clone()
    }
    pub fn set_uri(&self, uri:http::uri::Uri) {
	match self.data.write() {
	    Ok(mut x) => (*x).uri = Some(uri),
	    Err(e) => error!("Unable to lock websocket header struct: {}", e),
	}
    }
}
