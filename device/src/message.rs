use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

/// `Device`s communicate by sending and receiving `Message`s.
///
/// In this codebase, `Message`s are HTTP requests.
///
/// **Design Decision**: `headers` is purposefully not `pub` so that a `Message` cannot be created directly.
/// `Message`s must be created via one of the `impl` methods so that required headers can be added.
///
/// See: https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages
#[derive(PartialEq, Debug)]
pub struct Message {
    pub start_line: String,
    headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Allows `Message`s to be converted to `String`s with `to_string()`.
///
/// This implementation produces `String`s which conform to
/// [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#name-example-message-exchange).
///
/// **Design Decision**: when a `Message` is serialized, its `headers` are sorted alphabetically.
/// This makes it easier to make assertions on the serialized format of a message.
impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // sort headers so we can more easily make assertions about the serialized format
        let mut headers: Vec<(&String, &String)> = self.headers.iter().collect();
        headers.sort();

        let headers = headers.into_iter().map(|(key, value)| format!("{}: {}", key, value));
        let headers = headers.collect::<Vec<String>>().join("\r\n");

        // headers are always followed by a blank line, i.e. \r\n\r\n
        let headers = format!("{}\r\n", headers);

        let body = &self.body.as_ref().map(|b| format!("\r\n{}\r\n", b)).unwrap_or(String::from(""));
        write!(f, "{}\r\n{}{}\r\n", self.start_line.trim(), headers, body)
    }
}

impl Message {
    /// Creates a new `Message` from its constituent parts.
    ///
    /// **Design Decision**: this method is purposefully not `pub` so that a `Message` cannot be
    /// created directly. `Message`s must be created via one of the `pub` `impl` methods so that
    /// required headers can be added.
    ///
    /// **Design Decision**: by default, `Message`s all have their `Content-Type` set to `text/json`.
    /// Most messages are JSON blobs sent from one service to another. The `Content-Type` should
    /// be overridden to `text/html` when serving requests for HTML via the Web App.
    fn new(start_line: String, headers: HashMap<String, String>, body: Option<String>) -> Message {
        // All messages are JSON UTF-8.
        // Without this header, browsers will render "°C" as "Â°C"
        let mut headers = headers.clone();
        headers.insert("Content-Type".into(), "text/json; charset=utf-8".into());
        Message { start_line, headers, body }
    }

    /// Attempts to retrieve the specified header from this `Message`.
    ///
    /// This method is required because `headers` is purposefully not `pub`.
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Creates an arbitrary HTTP/1.1 request.
    ///
    /// **Design Decision**: this method is purposefully not `pub`. Users should instead use the
    /// `pub` `request_x` methods to construct HTTP requests of the required types.
    fn request(method: &str, url: &str) -> Message {
        let request_line = format!("{} {} HTTP/1.1", method, url);
        Message::new(request_line, HashMap::new(), None)
    }

    /// Creates a `GET` request against the specified `url`.
    pub fn request_get(url: &str) -> Message {
        Self::request("GET", url)
    }

    /// Creates a `POST` request against the specified `url`.
    pub fn request_post(url: &str) -> Message {
        Self::request("POST", url)
    }

    /// Creates an HTTP/1.1 response from its status `code`.
    ///
    /// **Design Decision**: this method is purposefully not `pub`. Users should instead use the
    /// `pub` `respond_x` methods to construct HTTP responses of the required types.
    fn respond(code: u16) -> Message {
        let text = match code {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            501 => "Not Implemented",
            _ => panic!("unexpected response code: {}", code),
        };

        let start_line = format!("HTTP/1.1 {} {}", code, text);
        Message::new(start_line, HashMap::new(), None)
    }

    /// Creates a simple `200 OK` response to acknowledge the successful handling of some request.
    pub fn respond_ok() -> Message {
        Self::respond(200)
    }

    /// Creates a `501 Not Implemented` response to indicate that we've not yet implemented some endpoint.
    pub fn respond_not_implemented() -> Message {
        Self::respond(501)
    }

    /// Creates a `400 Bad Request` response to indicate that the user has sent some request which are unable to handle.
    pub fn respond_bad_request() -> Message {
        Self::respond(400)
    }

    /// Creates a `404 Not Found` response to indicate that the user has requested some resource which does not exist.
    pub fn respond_not_found() -> Message {
        Self::respond(404)
    }

    /// Appends the given `headers` to this `Message`.
    pub fn with_headers(mut self, headers: HashMap<impl Into<String>, impl Into<String>>) -> Message {
        headers.into_iter().for_each(|(key, value)| {
            self.headers.insert(key.into(), value.into());
        });
        self
    }

    /// Sets the body of this `Message` to the provided `body`.
    pub fn with_body<S: Into<String>>(mut self, body: S) -> Message {
        let body = body.into();
        self.headers.insert("Content-Length".into(), body.len().to_string());
        self.body = Some(body);
        self
    }

    /// Writes this `Message` into the provided `tcp_stream`.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    pub fn write(&self, tcp_stream: &mut impl Write) {
        tcp_stream.write_all(self.to_string().as_bytes()).unwrap();
    }

    /// Attempts to read a `Message` from the provided `tcp_stream`.
    pub fn read(mut tcp_stream: &mut TcpStream) -> Result<Message, String> {
        Message::read_from_buffer(BufReader::new(&mut tcp_stream))
    }

    /// Attempts to read a `Message` from a `BufRead` (usually a `TcpStream`).
    ///
    /// **Design Decision**: similar to [`write`](Self::write), `tcp_stream` is of type `impl BufRead`
    /// rather than `TcpStream` because this is easier to test. [`read`](Self::read) is provided
    /// as well, for user convenience.
    fn read_from_buffer(mut tcp_stream: impl BufRead) -> Result<Message, String> {
        let mut message = String::new();
        tcp_stream.read_line(&mut message).map_err(|_| String::from("cannot read message"))?;

        let mut headers: HashMap<String, String> = HashMap::new();

        loop {
            let mut line = String::new();
            match tcp_stream.read_line(&mut line) {
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
            tcp_stream.read_exact(&mut buffer).unwrap();
            body = Some(std::str::from_utf8(buffer.as_slice()).unwrap().into());
        }

        let message = Message::new(String::from(message.trim()), headers, body);

        Ok(message)
    }
}

#[cfg(test)]
mod device_message_tests {
    use super::*;

    #[test]
    fn test_request_get() {
        let message = Message::request_get("/");
        let actual = message.to_string();

        let expected = ["GET / HTTP/1.1", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_request_post() {
        let message = Message::request_post("/");
        let actual = message.to_string();

        let expected = ["POST / HTTP/1.1", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_request_get_with_headers() {
        let message = Message::request_get("/");

        let mut headers = HashMap::new();
        headers.insert("foo", "bar");

        let message = message.with_headers(headers);
        let actual = message.to_string();

        let expected = ["GET / HTTP/1.1", "Content-Type: text/json; charset=utf-8", "foo: bar"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_request_get_header() {
        let message = Message::request_get("/");

        let mut headers = HashMap::new();
        headers.insert("foo", "bar");

        let message = message.with_headers(headers);

        let exists = message.header("foo");
        assert_eq!(exists, Some(String::from("bar")).as_ref());

        let does_not_exist = message.header("baz");
        assert_eq!(does_not_exist, None);
    }

    #[test]
    fn test_request_get_with_body() {
        let message = Message::request_get("/");

        let body = "Hello, World!";

        let message = message.with_body(body);
        let actual = message.to_string();

        let expected = [
            "GET / HTTP/1.1",
            "Content-Length: 13",
            "Content-Type: text/json; charset=utf-8",
            "",
            "Hello, World!",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_request_get_with_headers_with_body() {
        let message = Message::request_get("/");

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
            "",
            "Hello, World!",
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_respond_ok() {
        let message = Message::respond_ok();
        let actual = message.to_string();

        let expected = ["HTTP/1.1 200 OK", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_respond_bad_request() {
        let message = Message::respond_bad_request();
        let actual = message.to_string();

        let expected = ["HTTP/1.1 400 Bad Request", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_respond_not_found() {
        let message = Message::respond_not_found();
        let actual = message.to_string();

        let expected = ["HTTP/1.1 404 Not Found", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_respond_not_implemented() {
        let message = Message::respond_not_implemented();
        let actual = message.to_string();

        let expected = ["HTTP/1.1 501 Not Implemented", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    #[should_panic]
    fn test_respond_unknown_code() {
        Message::respond(999);
    }

    #[test]
    fn test_write() {
        let message = Message::respond_ok();

        let mut tcp_stream = Vec::new();
        message.write(&mut tcp_stream);
        let actual = String::from_utf8(tcp_stream).unwrap();

        let expected = ["HTTP/1.1 200 OK", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_read() {
        let expected = Message::respond_ok();

        let serialized = ["HTTP/1.1 200 OK", "Content-Type: text/json; charset=utf-8"].join("\r\n");

        let actual = Message::read_from_buffer(serialized.as_bytes()).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_read_with_misformatted_header() {
        let expected = Message::respond_ok();

        let serialized = [
            "HTTP/1.1 200 OK",
            "Content-Type: text/json; charset=utf-8",
            "kablooie", // this line is misformatted, it should be skipped
        ]
        .join("\r\n");

        let actual = Message::read_from_buffer(serialized.as_bytes()).unwrap();

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_read_with_body() {
        let message = Message::request_get("/");
        let body = "Hello, World!";
        let expected = message.with_body(body);

        let serialized = [
            "GET / HTTP/1.1",
            "Content-Length: 13",
            "Content-Type: text/json; charset=utf-8",
            "",
            "Hello, World!",
        ]
        .join("\r\n");

        let actual = Message::read_from_buffer(serialized.as_bytes()).unwrap();

        assert_eq!(actual, expected)
    }
}
