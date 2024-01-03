use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use actuator::Actuator;
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

use crate::command::Command;

pub mod command;

pub struct TemperatureActuator {
    id: Id,
    name: Name,
    environment: Arc<Mutex<Option<ServiceInfo>>>,
    address: Address,
}

impl Device for TemperatureActuator {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_model() -> Model {
        Model::Thermo5000
    }

    fn get_address(&self) -> Address {
        self.address
    }

    /// By default, an `Actuator` forwards all incoming requests to the `Environment`.
    fn get_handler(&self) -> Handler {
        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().clone();
        let environment = Arc::clone(self.get_environment());

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "POST /command HTTP/1.1" {
                    // send a Command to this Actuator (Command is in the body)
                    //     ex: curl 10.12.50.26:5454/command -d '{"name":"HeatTo","body":"25"}'

                    match message.body.as_ref().map(Command::parse) {
                        Some(Ok(command)) => {
                            let environment = environment.lock().unwrap();

                            match environment.as_ref().map(Self::extract_address) {
                                Some(address) => {
                                    println!("[Actuator] connecting to environment @ {}", address);
                                    let msg = format!("not yet implemented -- need to forward command {} to Environment", command);
                                    Self::handler_failure(self_name.clone(), stream, msg.as_str())

                                    // let mut stream = TcpStream::connect(address.to_string()).unwrap();
                                    // TODO actually send command to Environment
                                }
                                None => {
                                    let msg = "could not find environment";
                                    Self::handler_failure(self_name.clone(), stream, msg)
                                }
                            }
                        }
                        _ => {
                            let msg = format!("cannot parse body as command: {:?}", message.body);
                            Self::handler_failure(self_name.clone(), stream, msg.as_str())
                        }
                    }
                } else {
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
            } else {
                Self::handler_failure(self_name.clone(), stream, "unable to read Message from stream")
            }
        })
    }
}

impl Actuator for TemperatureActuator {
    fn new(id: Id, name: Name, address: Address) -> Self {
        Self {
            id,
            name,
            environment: Arc::new(Mutex::new(None)),
            address,
        }
    }

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.environment
    }

    fn get_environment_info(&self) -> Option<ServiceInfo> {
        let lock = self.environment.lock();
        let guard = lock.unwrap();
        guard.clone()
    }
}
