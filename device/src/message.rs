use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::BufRead;

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

        println!(
            "[parse_http_request] received\n==========\nmessage line: {}\nheaders: {:?}\nbody:\n----------\n{:?}\n==========",
            message.request_line.trim(),
            message.headers,
            message.body.as_ref().unwrap_or(&String::new())
        );

        Ok(message)
    }

    pub fn ping(sender: String) -> Message {
        let mut headers = HashMap::new();
        headers.insert("sender".into(), sender);

        Message::new("GET / HTTP/1.1", headers, None)
    }

    pub fn ack(sender: String) -> Message {
        let mut headers = HashMap::new();
        headers.insert("sender".into(), sender);

        Message::new("HTTP/1.1 200 OK", headers, None)
    }

    pub fn respond_ok(
        sender: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Message {
        let mut headers = headers.clone();
        headers.insert("sender".into(), sender);

        Message::new("HTTP/1.1 200 OK", headers, body)
    }

    pub fn respond_ok_with_body(
        sender: String,
        headers: HashMap<String, String>,
        body: &str,
    ) -> Message {
        let mut headers = headers.clone();
        headers.insert("Content-Length".into(), body.len().to_string());

        Self::respond_ok(sender, headers, Some(String::from(body)))
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
        let message = Message::ack("address".into());
        assert_eq!(
            message.to_string(),
            String::from("HTTP/1.1 200 OK\r\nsender: address\r\n\r\n")
        );
    }

    #[test]
    fn test_ok() {
        let headers = HashMap::new();
        let body = "Hello, World!";
        let message = Message::respond_ok("address".into(), headers, Some(body.into()));
        assert!(
            message.to_string() == *"HTTP/1.1 200 OK\r\nsender: address\r\nContent-Length: 13\r\n\r\nHello, World!\r\n\r\n" ||
                message.to_string() == *"HTTP/1.1 200 OK\r\nContent-Length: 13\r\nsender: address\r\n\r\nHello, World!\r\n\r\n"
        );
    }

    #[test]
    fn test_ok_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("key".into(), "value".into());
        let body = "Hello, World!";
        let message = Message::respond_ok("address".into(), headers, Some(body.into()));

        assert_eq!(message.headers.get("key"), Some(&String::from("value")));
        assert_eq!(
            message.headers.get("Content-Length"),
            Some(&String::from("13"))
        );
    }

    #[test]
    fn test_ok_with_body() {
        let headers = HashMap::new();
        let body = "Hello, World!";
        let message = Message::respond_ok_with_body("address".into(), headers, body);

        assert_eq!(
            message.headers.get("Content-Length"),
            Some(&String::from("13"))
        );
    }
}
