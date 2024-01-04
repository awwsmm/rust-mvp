use std::fmt::{Display, Formatter};
use std::net::IpAddr;

/// An `Address` contains all the information required to route a `Message` to a `Device`, namely
/// the `Device`'s IP address and port.
#[derive(Clone, Copy, PartialEq, Debug)]
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

#[cfg(test)]
mod device_address_tests {
    use super::*;

    #[test]
    fn test_display() {
        let ip = IpAddr::from([123, 234, 123, 255]);
        let address = Address::new(ip, 10101);

        let expected = "123.234.123.255:10101";
        let actual = address.to_string();

        assert_eq!(actual, expected)
    }
}
