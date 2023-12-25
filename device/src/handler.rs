use std::net::TcpStream;

/// A `Handler` defines how a `Device` should respond to incoming HTTP requests.
pub struct Handler {
    pub handle: fn(&mut TcpStream),
}

impl Handler {
    pub fn new(handle: fn(&mut TcpStream)) -> Handler {
        Handler { handle }
    }

    pub fn ignore() -> Handler {
        Handler { handle: |_| () }
    }
}
