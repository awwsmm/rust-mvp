use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, Write};
use std::net::TcpStream;

#[derive(PartialEq)]
pub struct Message {
    pub request_line: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Message {
    fn new(request_line: &str, headers: HashMap<String, String>, body: Option<String>) -> Message {
        let mut headers = headers.clone();

        if let Some(body) = body.as_ref() {
            headers.insert("Content-Length".into(), body.len().to_string());
        }

        Message {
            request_line: String::from(request_line),
            headers,
            body,
        }
    }

    pub fn from(mut reader: impl BufRead) -> Result<Message, String> {
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
                        Some((key, value)) => {
                            headers.insert(key.trim().into(), value.trim().into())
                        }
                        None => continue, // skip any header lines that can't be parsed
                    };
                }
                _ => break, // if the reader fails to read the next line, quit early
            };
        }

        let mut body: Option<String> = None;

        if let Some(length) = headers.get("Content-Length") {
            if let Ok(length) = length.parse::<usize>() {
                let mut buffer = vec![0; length];
                reader.read_exact(&mut buffer).unwrap();
                body = Some(std::str::from_utf8(buffer.as_slice()).unwrap().into());
            }
        }

        let message = Message::new(message.trim(), headers, body);

        // println!(
        //     "[parse_http_request] received\n==========\nmessage line: {}\nheaders: {:?}\nbody:\n----------\n{:?}\n==========",
        //     message.request_line.trim(),
        //     message.headers,
        //     message.body.as_ref().unwrap_or(&String::new())
        // );

        Ok(message)
    }

    pub fn ping_with_headers_and_body(
        sender_name: String,
        sender_address: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Message {
        let mut headers = headers.clone();
        headers.insert("sender_name".into(), sender_name);
        headers.insert("sender_address".into(), sender_address);

        Message::new("GET / HTTP/1.1", headers, body)
    }

    pub fn ping_with_body(
        sender_name: String,
        sender_address: String,
        body: Option<String>,
    ) -> Message {
        Self::ping_with_headers_and_body(sender_name, sender_address, HashMap::new(), body)
    }

    pub fn ping_with_headers(
        sender_name: String,
        sender_address: String,
        headers: HashMap<String, String>,
    ) -> Message {
        Self::ping_with_headers_and_body(sender_name, sender_address, headers, None)
    }

    pub fn ping(sender_name: String, sender_address: String) -> Message {
        Self::ping_with_headers_and_body(sender_name, sender_address, HashMap::new(), None)
    }

    pub fn ack(sender_name: String, sender_address: String) -> Message {
        let mut headers = HashMap::new();
        headers.insert("sender_name".into(), sender_name);
        headers.insert("sender_address".into(), sender_address);

        Message::new("HTTP/1.1 200 OK", headers, None)
    }

    pub fn respond_ok(
        sender_name: String,
        sender_address: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Message {
        let mut headers = headers.clone();
        headers.insert("sender_name".into(), sender_name);
        headers.insert("sender_address".into(), sender_address);

        Message::new("HTTP/1.1 200 OK", headers, body)
    }

    pub fn respond_ok_with_body(
        sender_name: String,
        sender_address: String,
        headers: HashMap<String, String>,
        body: &str,
    ) -> Message {
        let mut headers = headers.clone();
        headers.insert("Content-Length".into(), body.len().to_string());

        Self::respond_ok(
            sender_name,
            sender_address,
            headers,
            Some(String::from(body)),
        )
    }

    pub fn send(&self, tcp_stream: &mut TcpStream) {
        tcp_stream.write_all(self.to_string().as_bytes()).unwrap();
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let headers = self.headers.iter().map(|(k, v)| format!("{}: {}", k, v));
        let headers = headers.collect::<Vec<String>>().join("\r\n");
        let headers = if headers.is_empty() {
            String::from("")
        } else {
            format!("{}\r\n", headers)
        };
        let body = &self
            .body
            .as_ref()
            .map(|b| format!("\r\n{}\r\n", b))
            .unwrap_or(String::from(""));
        write!(f, "{}\r\n{}{}\r\n", self.request_line.trim(), headers, body)
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn test_ack() {
        let sender_name = "My Device";
        let sender_address = "123.234.210.123:12345";
        let message = Message::ack(sender_name.into(), sender_address.into());

        let serialized = message.to_string();

        assert!(serialized.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(serialized.ends_with("\r\n\r\n"));

        assert!(serialized.contains("\r\nsender_name: My Device\r\n"));
        assert!(serialized.contains("\r\nsender_address: 123.234.210.123:12345\r\n"));
    }

    #[test]
    fn test_ok() {
        let sender_name = "My Device";
        let sender_address = "123.234.210.123:12345";
        let headers = HashMap::new();
        let body = "Hello, World!";
        let message = Message::respond_ok(
            sender_name.into(),
            sender_address.into(),
            headers,
            Some(body.into()),
        );

        let serialized = message.to_string();

        assert!(serialized.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(serialized.ends_with("\r\n\r\n"));

        assert!(serialized.contains("\r\nsender_name: My Device\r\n"));
        assert!(serialized.contains("\r\nsender_address: 123.234.210.123:12345\r\n"));

        assert!(serialized.contains("\r\n\r\nHello, World!\r\n\r\n"));
    }

    #[test]
    fn test_ok_with_headers() {
        let sender_name = "My Device";
        let sender_address = "123.234.210.123:12345";
        let mut headers = HashMap::new();
        headers.insert("key".into(), "value".into());
        let body = "Hello, World!";
        let message = Message::respond_ok(
            sender_name.into(),
            sender_address.into(),
            headers,
            Some(body.into()),
        );

        let serialized = message.to_string();

        assert!(serialized.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(serialized.ends_with("\r\n\r\n"));

        assert!(serialized.contains("\r\nsender_name: My Device\r\n"));
        assert!(serialized.contains("\r\nsender_address: 123.234.210.123:12345\r\n"));
        assert!(serialized.contains("\r\nkey: value\r\n"));

        assert!(serialized.contains("\r\n\r\nHello, World!\r\n\r\n"));
    }

    #[test]
    fn test_ok_with_body() {
        let sender_name = "My Device";
        let sender_address = "123.234.210.123:12345";
        let headers = HashMap::new();
        let body = "Hello, World!";
        let message =
            Message::respond_ok_with_body(sender_name.into(), sender_address.into(), headers, body);

        let serialized = message.to_string();

        assert!(serialized.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(serialized.ends_with("\r\n\r\n"));

        assert!(serialized.contains("\r\nsender_name: My Device\r\n"));
        assert!(serialized.contains("\r\nsender_address: 123.234.210.123:12345\r\n"));

        assert!(serialized.contains("\r\n\r\nHello, World!\r\n\r\n"));
    }
}
