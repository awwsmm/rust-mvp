use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(PartialEq)]
pub struct Message {
    pub request_line: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Message {
    pub fn new(
        request_line: &str,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Message {
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

    pub fn ack() -> Message {
        Message::new("HTTP/1.1 200 OK", HashMap::new(), None)
    }

    pub fn respond_ok(headers: HashMap<String, String>, body: Option<String>) -> Message {
        Message::new("HTTP/1.1 200 OK", headers, body)
    }

    pub fn respond_ok_with_body(headers: HashMap<String, String>, body: &str) -> Message {
        let mut headers = headers.clone();
        headers.insert("Content-Length".into(), body.len().to_string());
        Self::respond_ok(headers, Some(String::from(body)))
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let headers = self.headers.iter().map(|(k, v)| format!("{}: {}", k, v));
        let headers = headers.collect::<Vec<String>>().join("\r\n");
        let body = &self
            .body
            .as_ref()
            .map(|b| format!("\r\n\r\n{}\r\n", b))
            .unwrap_or(String::from(""));
        write!(f, "{}\r\n{}{}\r\n", self.request_line, headers, body)
    }
}

#[cfg(test)]
mod message_tests {
    use super::*;

    #[test]
    fn test_ack() {
        let message = Message::ack();
        assert_eq!(message.to_string(), String::from("HTTP/1.1 200 OK\r\n\r\n"));
    }

    #[test]
    fn test_ok() {
        let headers = HashMap::new();
        let body = "Hello, World!";
        let message = Message::respond_ok(headers, Some(body.into()));
        assert_eq!(
            message.to_string(),
            String::from("HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello, World!\r\n\r\n")
        );
    }

    #[test]
    fn test_ok_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("key".into(), "value".into());
        let body = "Hello, World!";
        let message = Message::respond_ok(headers, Some(body.into()));

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
        let message = Message::respond_ok_with_body(headers, body);

        assert_eq!(
            message.headers.get("Content-Length"),
            Some(&String::from("13"))
        );
    }
}
