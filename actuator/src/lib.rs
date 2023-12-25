use std::fmt::Display;
use std::io::Write;
use std::net::TcpStream;

use device::message::Message;
use device::Device;

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    // /// The `act` command tells the actuator to perform some action.
    // ///
    // /// The action can be anything (turning on a light, setting a thermostat target temperature to
    // /// some value, locking a door, etc.), so the `command` is a `String` which can be formatted in
    // /// any way by the `Controller` and parsed in any way by the `Actuator`.
    // ///
    // /// In the "real world", this would perform some actual, physical action.
    // ///
    // /// In our example MVP, this sends a command to the `Environment` which mutates its state.
    // fn act(&self, device: Id, command: String);

    /// Responds to all incoming requests by forwarding them to the `Environment`.
    fn handle(stream: &mut TcpStream) {
        if let Ok(message) = Self::parse_http_request(stream) {
            println!(
                "[handle] actuator received\n==========\nrequest line: {}\nheaders: {:?}\nbody:\n----------\n{:?}\n==========",
                message.request_line.trim(),
                message.headers,
                message.body.unwrap_or_default()
            );

            stream
                .write_all(Message::ack().to_string().as_bytes())
                .unwrap();

            // TODO forward command to Environment
        }

        // // TODO Does Id matter since the appropriate Actuator is handling it anyways?
        // let temp_id = Id::new(&Uuid::new_v4().to_string());
        //
        // // TODO Should we add a return type to `act` and then our HTTP response
        // //  depends on the success of the act call? Or just always send back 200?
        // self.act(temp_id, body.to_string());
    }
}

pub trait Command: Display {}
