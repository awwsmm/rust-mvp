use std::net::TcpStream;
use std::sync::Arc;

use mdns_sd::ServiceDaemon;

/// A `Handler` defines how a `Device` should respond to incoming HTTP requests.
pub struct Handler {
    pub handle: fn(&mut TcpStream, Arc<ServiceDaemon>),
}

impl Handler {
    pub fn new(handle: fn(&mut TcpStream, Arc<ServiceDaemon>)) -> Handler {
        Handler { handle }
    }

    pub fn ignore() -> Handler {
        Handler { handle: |_, _| () }
    }
}
