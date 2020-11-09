use tungstenite::handshake::server::{Callback,ErrorResponse,Request,Response};
use std::sync::Arc;
use log::debug;
#[derive(Debug)]
pub struct WebsocketHeaders {
    pub uri: Option<http::uri::Uri>,
}

pub struct WebsocketHeadersCB {
    data: Arc<WebsocketHeaders>,
}

impl Callback for WebsocketHeadersCB {
    fn on_request(self, request: &Request, response: Response)
		  -> Result<Response, ErrorResponse> {
	self.data.borrow().set_uri(request.uri().clone()); 
	debug!("URI: {}", request.uri());
	Ok(response)
    }
}

pub fn new_callback() -> WebsocketHeadersCB {
    WebsocketHeadersCB {
	data: Arc::new(
	    WebsocketHeaders {
		uri: None,
	    })
    }
}

impl WebsocketHeadersCB {
    pub fn hdr(&self) -> Arc<WebsocketHeaders> {
	self.data.clone()
    }
    pub fn set_uri(&self, uri:http::uri::Uri) {
	self.data.uri = Some(uri);
    }
}
