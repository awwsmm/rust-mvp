use std::fmt::{Display, Formatter};
use std::net::IpAddr;

#[derive(Clone, Copy)]
pub struct Address {
    ip: IpAddr,
    port: u16,
}

impl Address {
    pub fn new(ip: IpAddr, port: u16) -> Address {
        Address { ip, port }
    }
}

/// Allows `Address`es to be converted to `String`s with `to_string()`.
impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}
