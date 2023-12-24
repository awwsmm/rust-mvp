use std::net::TcpStream;

pub struct Handler {
    pub handle: fn(&mut TcpStream),
}

impl Handler {
    pub fn new(handle: fn(&mut TcpStream)) -> Handler {
        Handler { handle }
    }
}
