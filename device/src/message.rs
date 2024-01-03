use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use crate::address::Address;

/// `Device`s communicate by sending and receiving `Message`s.
///
/// **Design Decision**: in this codebase, `Message`s are HTTP requests. All communication happens asynchronously via
/// "fire and forget" HTTP requests (all responses to all messages are "200 OK"). This _asynchronous message-passing_
/// style of communication is the de-facto standard in
/// [microservices design](https://docs.aws.amazon.com/whitepapers/latest/microservices-on-aws/asynchronous-messaging-and-event-passing.html).
///
/// **Design Decision**: `request_line` is purposefully not `pub` so that a `Message` cannot be created directly.
/// `Message`s must be created via one of the `impl` methods so that required headers can be added.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages
#[derive(PartialEq, Debug)]
pub struct Message {
    pub start_line: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Allows `Message`s to be converted to `String`s with `to_string()`.
///
/// This implementation produces `String`s which conform to
/// [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#name-example-message-exchange).
impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // sort headers so we can more easily make assertions about the serialized format
        let mut headers: Vec<(&String, &String)> = self.headers.iter().collect();
        headers.sort();

        let headers = headers.into_iter().map(|(key, value)| format!("{}: {}", key, value));
        let headers = headers.collect::<Vec<String>>().join("\r\n");

        // headers are always followed by a blank line, i.e. \r\n\r\n
        let headers = format!("{}\r\n", headers);

        let body = &self
            .body
            .as_ref()
            .map(|b| format!("\r\n{}\r\n", b))
            .unwrap_or(String::from(""));
        write!(f, "{}\r\n{}{}\r\n", self.start_line.trim(), headers, body)
    }
}

impl Message {
    /// Creates a new `Message` from its constituent parts.
    ///
    /// **Design Decision**: this method is purposefully not `pub` so that a `Message` cannot be
    /// created directly. `Message`s must be created via one of the `pub` `impl` methods so that
    /// required headers can be added.
    fn new(request_line: &str, headers: HashMap<String, String>, body: Option<String>) -> Message {
        // All messages are JSON UTF-8.
        // Without this header, browsers will render "°C" as "Â°C"
        let mut headers = headers.clone();
        headers.insert("Content-Type".into(), "text/json; charset=utf-8".into());

        Message {
            start_line: String::from(request_line),
            headers,
            body,
        }
    }

    /// Creates a simple `GET` message to ping one `Device` from another.
    pub fn ping(sender_name: &str, sender_address: Address) -> Message {
        let mut headers = HashMap::new();
        headers.insert("sender_name".into(), sender_name.into());
        headers.insert("sender_address".into(), sender_address.to_string());
        Message::new("GET / HTTP/1.1", headers, None)
    }

    /// Adds the given `headers` to this `Message`.
    pub fn with_headers(mut self, headers: HashMap<impl Into<String>, impl Into<String>>) -> Message {
        headers.into_iter().for_each(|(key, value)| {
            self.headers.insert(key.into(), value.into());
        });
        self
    }

    /// Sets the body of this message to the provided `body`.
    pub fn with_body<S: Into<String>>(mut self, body: S) -> Message {
        let body = body.into();
        self.headers.insert("Content-Length".into(), body.len().to_string());
        self.body = Some(body);
        self
    }

    /// Creates a simple `200 OK` response to acknowledge the receipt of a `Message`.
    pub fn ack(sender_name: &str, sender_address: Address) -> Message {
        let mut message = Message::ping(sender_name, sender_address);
        message.start_line = "HTTP/1.1 200 OK".into();
        message
    }

    pub fn request(method: &str, url: &str) -> Message {
        let request_line = format!("{} {} HTTP/1.1", method, url);
        Message::new(request_line.as_str(), HashMap::new(), None)
    }

    pub fn respond_ok() -> Message {
        Message::new("HTTP/1.1 200 OK", HashMap::new(), None)
    }

    pub fn respond_not_implemented() -> Message {
        Message::new("HTTP/1.1 501 Not Implemented", HashMap::new(), None)
    }

    pub fn respond_bad_request() -> Message {
        Message::new("HTTP/1.1 400 Bad Request", HashMap::new(), None)
    }

    pub fn respond_not_found() -> Message {
        Message::new("HTTP/1.1 404 Not Found", HashMap::new(), None)
    }

    /// Writes this `Message` into the provided `tcp_stream`.
    pub fn write(&self, tcp_stream: &mut impl Write) {
        tcp_stream.write_all(self.to_string().as_bytes()).unwrap();
    }

    pub fn read(mut stream: &mut TcpStream) -> Result<Message, String> {
        Message::read_from_buffer(BufReader::new(&mut stream))
    }

    /// Attempts to read a `Message` from a `BufRead` (usually a `TcpStream`).
    pub fn read_from_buffer(mut reader: impl BufRead) -> Result<Message, String> {
        let mut message = String::new();
        reader
            .read_line(&mut message)
            .map_err(|_| String::from("cannot read message"))?;

        let mut headers: HashMap<String, String> = HashMap::new();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(size) if size > 2 => {
                    // a blank line (CRLF only) separates HTTP headers and body
                    match line.split_once(": ") {
                        // HTTP headers are always formatted as "key: value"
                        Some((key, value)) => headers.insert(key.trim().into(), value.trim().into()),
                        None => continue, // skip any header lines that can't be parsed
                    };
                }
                _ => break, // if the reader fails to read the next line, quit early
            };
        }

        let mut body: Option<String> = None;

        // we write the Content-Length header, so we can assume it's correctly formatted
        if let Some(length) = headers.get("Content-Length") {
            let length = length.parse::<usize>().unwrap();
            let mut buffer = vec![0; length];
            reader.read_exact(&mut buffer).unwrap();
            body = Some(std::str::from_utf8(buffer.as_slice()).unwrap().into());
        }

        let message = Message::new(message.trim(), headers, body);

        Ok(message)
    }
}

#[cfg(test)]
mod device_message_tests {
    use std::net::IpAddr;

    use super::*;

    #[test]
    fn test_ping() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ping(sender_name, sender_address);
        let actual = message.to_string();

        let expected = [
            "GET / HTTP/1.1",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_ping_with_headers() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ping(sender_name, sender_address);

        let mut headers = HashMap::new();
        headers.insert("foo", "bar");

        let message = message.with_headers(headers);
        let actual = message.to_string();

        let expected = [
            "GET / HTTP/1.1",
            "Content-Type: text/json; charset=utf-8",
            "foo: bar",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_ping_with_body() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ping(sender_name, sender_address);

        let body = "Hello, World!";

        let message = message.with_body(body);
        let actual = message.to_string();

        let expected = [
            "GET / HTTP/1.1",
            "Content-Length: 13",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
            "",
            "Hello, World!",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_ping_with_headers_with_body() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ping(sender_name, sender_address);

        let mut headers = HashMap::new();
        headers.insert("foo", "bar");
        let body = "Hello, World!";

        let message = message.with_headers(headers).with_body(body);
        let actual = message.to_string();

        let expected = [
            "GET / HTTP/1.1",
            "Content-Length: 13",
            "Content-Type: text/json; charset=utf-8",
            "foo: bar",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
            "",
            "Hello, World!",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_ack() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ack(sender_name, sender_address);
        let actual = message.to_string();

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_write() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ack(sender_name, sender_address);

        let mut tcp_stream = Vec::new();
        message.write(&mut tcp_stream);
        let actual = String::from_utf8(tcp_stream).unwrap();

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_read() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let expected = Message::ack(sender_name, sender_address);

        let serialized = [
            "HTTP/1.1 200 OK",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
        ]
        .join("\r\n");

        let actual = Message::read_from_buffer(serialized.as_bytes()).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_read_with_misformatted_header() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let expected = Message::ack(sender_name, sender_address);

        let serialized = [
            "HTTP/1.1 200 OK",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
            "kablooie", // this line is misformatted, it should be skipped
        ]
        .join("\r\n");

        let actual = Message::read_from_buffer(serialized.as_bytes()).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_read_with_body() {
        let sender_name = "My Device";
        let sender_address = Address::new(IpAddr::from([123, 234, 210, 123]), 12345);

        let message = Message::ping(sender_name, sender_address);
        let body = "Hello, World!";
        let expected = message.with_body(body);

        let serialized = [
            "GET / HTTP/1.1",
            "Content-Length: 13",
            "Content-Type: text/json; charset=utf-8",
            "sender_address: 123.234.210.123:12345",
            "sender_name: My Device",
            "",
            "Hello, World!",
        ]
        .join("\r\n");

        let actual = Message::read_from_buffer(serialized.as_bytes()).unwrap();

        assert_eq!(actual, expected)
    }
}
