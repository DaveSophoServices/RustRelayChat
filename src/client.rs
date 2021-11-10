use std::net::{SocketAddr,TcpStream};
use std::sync::{Arc,Mutex,RwLock,mpsc};
use tungstenite::{accept_hdr,Error,Message,WebSocket};
use std::thread::spawn;
use std::time::Duration;

#[cfg(feature="dblog")]
use crate::dblog::{logmessage, logmessage::LogMessage};
use crate::server::channel_server::ChannelServer;
use crate::stats::Stats;
use crate::server::Server;
use crate::websocket_headers;
use crate::hasher;
use crate::userinfo;
use log::{debug,info,warn,error};

pub struct Client {
    userinfo: Mutex<userinfo::UserInfo>,
    addr: SocketAddr,
    websocket_ro: Mutex<WebSocket<TcpStream>>,
    websocket_wo: Mutex<WebSocket<TcpStream>>,
    ch: Arc<ChannelServer>,
    pair_shutdown: Arc<RwLock<u32>>,
    shutdown: Arc<RwLock<u32>>,
    stats: Arc<Stats>,
    tx: Arc<Mutex<mpsc::Sender<Message>>>,
    rx: Arc<Mutex<mpsc::Receiver<Message>>>,
    main_server: Arc<Server>,
    #[cfg(feature="dblog")]
    log_channel: Option<Mutex<mpsc::Sender<LogMessage>>>,
}

pub fn new(stream: TcpStream, main_server: Arc<Server>) -> Option<Arc<Client>> {
    let ws_hdr_cb = websocket_headers::new_callback();
    let ws_hdr = ws_hdr_cb.hdr();
    let addr = stream.peer_addr().unwrap();

    //TODO: how to handle a TlsStream and TcpStream with the same code?
    if main_server.has_tls() {
            match main_server.negotiate_tls(stream) {
                Ok(s) => s,
                Err(err) => { 
                    warn!("Unable to negotiate TLS with {}: {}", addr, err)
                }
            }
        } else {
            stream
        });


    let stream_clone = stream.try_clone();
    let websocket_wo =
    Mutex::new(WebSocket::from_raw_socket(
        stream_clone.unwrap(),
        tungstenite::protocol::Role::Server,
        None
    ));
    
    let websocket_ro = Mutex::new(accept_hdr(
        stream, ws_hdr_cb
    ).unwrap());
    
    let ch = match main_server.get(ws_hdr.clone()) {
        Some(x) => Arc::new(x),
        None => {
            warn!("[{}] tried to create channel {:?} but not allowed", addr, ws_hdr);
            return None;
        },
    };
    
    let (tx,rx) = ch.get_tx_rx();
    
    let stats = ch.get_stats();
    
    #[cfg(feature="dblog")]
    {
        let log_channel =
        match main_server.logger_channel() {
            Some(lc) => Some(Mutex::new(lc)),
            None => None,
        };
    }
    let r = Arc::new(Client {
        userinfo: Mutex::new(userinfo::UserInfo::blank()),
        addr,
        websocket_ro,
        websocket_wo,
        ch,
        rx: Arc::new(Mutex::new(rx)),
        tx: Arc::new(Mutex::new(tx)),
        pair_shutdown: Arc::new(RwLock::new(0)),
        shutdown: main_server.shutdown_ref(),
        stats,
        main_server,
        #[cfg(feature="dblog")]
        log_channel,
    });
    info!("new connection: {}", r.addr);
    r.ch.add_client(r.clone());
    
    // spin off the threads to do the receiving and sending
    sender(r.clone());
    receiver(r.clone());
    
    return Some(r);	
}


// central -> webbrowser socket
fn sender(client: Arc<Client>) {
    let channel_read_duration = Duration::from_secs(1);
    
    spawn(move || {
        // WRITE Loop
        let mut old_stats_version: u32 = 0xFFFFFFFF;
        loop {
            if client.check_shutdowns() != 0 {
                debug!("[{}] write loop shutdown requested", client.addr);
                break;
            }
            if client.stats.ver() != old_stats_version {
                old_stats_version = client.stats.ver();
                client.write(client.stats.stat_msg());
            }
            
            // check if anything from central
            if let Ok(rx) = client.rx.lock() {
                match rx.recv_timeout(channel_read_duration) {
                    Ok(msg) => {
                        if let Message::Ping(_) = msg {
                            // ignore it. Just central checking we're alive
                        } else {
                            client.write(msg);
                        }
                    },
                    Err(mpsc::RecvTimeoutError::Timeout) => (), // ignore it
                    Err(mpsc::RecvTimeoutError::Disconnected) => 
                    client.mark_connection_closed(),
                }
            }	
        }
        debug!("[{}] closed write loop", client.addr);
    });
}

// webbrowser socket -> central
fn receiver(client: Arc<Client>) {
    spawn(move || {
        loop { 
            if client.check_shutdowns() != 0 {
                debug!("[{}] closing read loop due to client shutdown req",
                client.addr);
                break;
            }
            if let Ok(mut ws) = client.websocket_ro.lock() {
                match ws.read_message() {
                    Ok(Message::Text(msg)) => {
                        // going to log the command
                        #[cfg(feature="dblog")]
                        client.log(&msg);
                        let mut handled = false;
                        if msg.starts_with('/') {
                            debug!("[{}] {} command", client.addr, msg);
                            // split off the command
                            let c:Vec<&str> = msg.splitn(2, ' ').collect();
                            match c[0] {
                                "/QUIT" => {
                                    debug!("[{}] Going to close connection",
                                        client.addr);
                                    client.close("** Going to close connection.");
                                },
                                "/USER" => {
                                    debug!("[{}] Setting user info", client.addr);
                                    match client.set_info(c[1]) {
                                        Ok(_) => (),
                                        Err(e) => {
                                            // we need to close this connection
                                            error!("[{}] unable to set client info: {}",
                                                    client.addr, e);
                                            client.write(Message::Text("!*MSG Please login to citysaver before connecting to chat.".to_string()));
                                            client.close("** Going to close connection.");
                                        },
                                    }
                                },
                                "/USERS" => {
                                    // list the users
                                    client.write(Message::from(
                                        client.get_userlist()));
                                }
                                _ => {
                                    warn!("[{}] unknown command: {:?}", client.addr, c);
                                }
                            }
                            handled = true;
                        }
                        
                        if !handled {
                            // prepend the originating user's name
                            let mut msg = format!("{}: {}", client.get_name(), msg);
                            if msg.starts_with("!*") {
                                msg.insert(0, ' ');
                            }
                            debug!("[{}] Sending msg ({:?}) to central", client.addr, msg);
                            client.to_central(msg);
                        }
                    }	    
                    Ok(_) => (), // ignore other websocket message types
                    Err(Error::ConnectionClosed) => {
                        info!("[{}] websocket closed.", client.addr);
                        client.mark_connection_closed();
                    },
                    Err(Error::AlreadyClosed) => {
                        info!("[{}] websocket already closed.", client.addr);
                        client.mark_connection_closed();
                    },	    
                    Err(e) => {
                        info!("[{}] websocket error: ({}) {}",
                        client.addr, type_of(&e), e);
                        client.mark_connection_closed();
                    },
                }
            }
        } // end of loop
        debug!("[{}] closed read loop.", client.addr);
    });
}

fn type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.addr)
    }
}
impl Client {
    fn check_shutdowns(&self) -> u32 {
        if let Ok(i) = self.shutdown.read() {
            if *i != 0 {
                return *i;
            }
        }
        if let Ok(i) = self.pair_shutdown.read() {
            if *i != 0 {
                return *i;
            }
        }
        return 0;
    }
    
    fn close(&self, msg: &str) {
        self.write(Message::Text(msg.to_string()));
        self.write(Message::Close(None));
        self.mark_connection_closed();
    }
    
    pub fn get_name(&self) -> String {
        match self.userinfo.lock() {
            Ok(s) => s.display.clone(),
            Err(e) => panic!("[{}] Unable to obtain lock for name: {}",
                                self.addr, e),
        }
    }

    fn get_clone_of_userinfo(&self) -> userinfo::UserInfo {
        match self.userinfo.lock() {
            Ok(s) => s.clone(),
            Err(e) => panic!("[{}] Unable to obtain lock for userinfo: {}", self.addr, e),
        }
    }

    fn set_userinfo(&self, user:userinfo::UserInfo) {
        match self.userinfo.lock() {
            Ok(mut u) => *u = user,
            Err(e) => panic!("[{}] Unable to obtain lock for set_userinfo: {}", self.addr, e),
        }
    }

    fn get_userlist(&self) -> String {
        // asks the channel for a list of usernames connected
        self.ch.get_userlist()
    }

    // record the incoming message (&String) to the database
    #[cfg(feature="dblog")]
    fn log(&self, msg: &str) {
        let user = self.get_clone_of_userinfo();
        debug!("Going to log {} by {} to database", user.display, msg);
        match &self.log_channel {
            Some(ch) => {
                if let Ok(ch) = ch.lock() {
                    match ch.send(logmessage::new(user,
                        self.addr.clone(),
                        self.ch.get_name(),
                        msg.to_string())) {
                            Ok(_) => (),
                            Err(e) =>
                            error!("Unable to send LogMessage for logging: {}",e),
                        
                    }
                }
            },
            None => (),
        }
    }
    
    // called when we have an error that wants us to terminate
    fn mark_connection_closed(&self) {
        if let Ok(mut i) = self.pair_shutdown.write() {
            debug!("Marking our connection pair as closing.");
            *i = 1;
        }
        self.ch.remove_client(&self);
    }    
    
    
    fn set_info(&self, arg: &str) -> Result<bool, String> {
        // check the hmac at the end first
        let a:Vec<&str> = arg.rsplitn(2,'\n').collect();
        match hasher::verify(a[0], a[1], self.main_server.get_secret_key()) {
            Ok(_) => { 
                // yes, the info is good
                debug!("info is good: {:?}", a[1]);
                let info = userinfo::UserInfo::new(a[1]);
                if let Some(info) = info {
                    if info.err != "" {
                        Err(info.err)
                    } else {
                        // assign the details to our session
                        self.set_userinfo(info);
                        Ok(true)
                    }
                } else {
                    Err("Failed to decode user info".to_string())
                }
            },
            Err(e) => {
                // no, the hmac is incorrect
                error!("[{}] bad info signature: {}", self.addr, e);
                Err(e)
            }
        }
        
    }
    
    fn to_central(&self, msg: String) {
        if let Ok(tx) = self.tx.lock() {
            if let Err(e) = tx.send(Message::Text(msg)) {
                error!("[{}] unable to send msg to central: {}",
                self.addr, e);
            }
        }
        
    }
    
    fn write(&self, msg: Message) {
        if let Ok(mut ws) = self.websocket_wo.lock() {
            match ws.write_message(msg) {
                Err(Error::ConnectionClosed) => self.mark_connection_closed(),
                Err(e) => {
                    // we got a fatal error from the connection
                    // it's probably died
                    debug!("[{}] shutdown due to websocket error: {}",
                    self.addr, e);
                    self.mark_connection_closed();
                },
                Ok(_) => (),
            }
        }
    }
}
