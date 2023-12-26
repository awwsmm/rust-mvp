use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::ServiceInfo;

use crate::handler::Handler;
use crate::id::Id;
use crate::message::Message;
use crate::model::Model;
use crate::name::Name;

pub mod handler;
pub mod id;
pub mod message;
pub mod model;
pub mod name;

/// A `Device` exists on the network and is discoverable via mDNS.
pub trait Device {
    /// Returns the user-friendly name of this `Device`.
    fn get_name(&self) -> &Name;

    /// Returns the unique ID of this `Device`.
    fn get_id(&self) -> &Id;

    /// Returns the model of this `Device`, which may or may not be supported by the `Controller`.
    fn get_model() -> Model;

    /// Returns the mDNS group of this `Device`, i.e. "_sensor", "_actuator", etc.
    fn get_group() -> String;

    /// Returns the full mDNS name of this `Device` (e.g. "id._model._group._tcp.local.")
    fn get_fullname(&self) -> String {
        format!(
            "{}.{}.{}._tcp.local.",
            self.get_id(),
            Self::get_model().id(),
            Self::get_group()
        )
    }

    /// Returns the helper which defines how to handle HTTP requests.
    fn get_handler(&self) -> Handler;

    /// Registers this `Device` with mDNS in the specified group.
    fn register(&self, ip: IpAddr, port: u16, group: &str) {
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();
        let host = ip.clone().to_string();
        let name = format!("{}.{}", self.get_id(), Self::get_model().id());
        let domain = format!("{}._tcp.local.", group);

        println!("Registering new device via mDNS at {}.{}", name, domain);

        let mut properties = HashMap::new();
        properties.insert(String::from("id"), self.get_id().to_string());
        properties.insert(String::from("model"), Self::get_model().id());

        let my_service = ServiceInfo::new(
            domain.as_str(),
            name.as_str(),
            host.as_str(),
            ip,
            port,
            properties,
        )
        .unwrap();

        mdns.register(my_service).unwrap()
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn bind(&self, ip: IpAddr, port: u16) -> TcpListener {
        let host = ip.clone().to_string();
        let address = format!("{}:{}", host, port);
        let name = &self.get_name();

        println!("Creating new device '{}' at {}", name, address);

        TcpListener::bind(address).unwrap()
    }

    /// Reads a message from a `TcpStream` and parses it into the message line, headers, and body.
    fn parse_http_request(stream: &TcpStream) -> Result<Message, String> {
        let mut reader = BufReader::new(stream);

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

    /// `register`s and `bind`s this `Device`, then spawns a new thread where it will continually
    /// listen for incoming `TcpStream`s and handles them appropriately.
    fn respond(&self, ip: IpAddr, port: u16, group: &str) -> JoinHandle<()> {
        self.register(ip, port, group);
        let listener = self.bind(ip, port);
        let handler = self.get_handler();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                (handler.handle)(&mut stream);
            }
        })
    }

    /// Configures this `Device` to `respond` to incoming requests and discover `targets` for outgoing requests.
    fn run(
        &self,
        ip: IpAddr,
        port: u16,
        group: &str,
        targets: HashMap<String, &Arc<Mutex<HashMap<Id, ServiceInfo>>>>,
    ) -> JoinHandle<()> {
        for (group, devices) in targets.iter() {
            self.discover(group, devices);
        }
        self.respond(ip, port, group)
    }

    fn extract_id(info: &ServiceInfo) -> Option<Id> {
        let id = info.get_property("id").map(|p| p.to_string());
        id.map(|i| Id::new(i.trim_start_matches("id=")))
    }

    fn extract_model(info: &ServiceInfo) -> Option<Result<Model, String>> {
        let model = info.get_property("model").map(|p| p.to_string());
        model.map(|m| Model::parse(m.trim_start_matches("model=")))
    }

    /// Creates a new thread to continually discover `Device`s on the network in the specified group.
    fn discover(
        &self,
        group: &str,
        devices: &Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    ) -> JoinHandle<()> {
        let group = String::from(group);

        // clone the Arc<Mutex<>> around the devices so we can update them in multiple threads
        let devices_mutex = Arc::clone(devices);

        let self_fullname = self.get_fullname().clone();

        std::thread::spawn(move || {
            let mdns = mdns_sd::ServiceDaemon::new().unwrap();
            let service_type = format!("{}._tcp.local.", group);
            let receiver = mdns.browse(service_type.as_str()).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let id = Self::extract_id(&info);
                    let devices_lock = devices_mutex.lock();
                    let mut devices_guard = devices_lock.unwrap();

                    println!(
                        "[Device::discover] {} discovered {}",
                        self_fullname,
                        info.get_fullname()
                    );

                    id.map(|i| devices_guard.insert(i, info));
                }
            }
        })
    }
}
