use std::io::Read;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use uuid::Uuid;

use device::{Device, Id};

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    /// The `act` command tells the actuator to perform some action.
    ///
    /// The action can be anything (turning on a light, setting a thermostat target temperature to
    /// some value, locking a door, etc.), so the `command` is a `String` which can be formatted in
    /// any way by the `Controller` and parsed in any way by the `Actuator`.
    ///
    /// In the "real world", this would perform some actual, physical action.
    ///
    /// In our example MVP, this sends a command to the `Environment` which mutates its state.
    fn act(&self, device: Id, command: String);

    /// Responds to all incoming requests by forwarding them to the `Environment`.
    fn respond(&self, listener: TcpListener) {
        for mut stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(&mut stream);
            let mut request = String::new();
            let mut content_length: usize = 0;

            // Read the headers
            loop {
                let mut line = String::new();
                let len = reader.read_line(&mut line).unwrap();
                if len == 0 || line == "\r\n" {
                    break;
                }

                if line.starts_with("Content-Length:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    content_length = parts[1].parse().unwrap_or(0);
                }

                request.push_str(&line);
            }

            // Read the body based on Content-Length
            let mut body = String::new();
            if content_length > 0 {
                let mut body_buffer = vec![0; content_length];
                reader.read_exact(&mut body_buffer).unwrap();
                body = String::from_utf8(body_buffer).unwrap_or_default();
            }

            println!(
                "{} received request: {} with body: {}",
                self.get_name(),
                request.trim(),
                &body
            );

            // TODO Does Id matter since the appropriate Actuator is handling it anyways?
            let temp_id = Id::new(&Uuid::new_v4().to_string());

            // TODO Should we add a return type to `act` and then our HTTP response
            //  depends on the success of the act call? Or just always send back 200?
            self.act(temp_id, body.to_string());

            let ack = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(ack.as_bytes()).unwrap();
        }
    }
}

pub trait Command {}
