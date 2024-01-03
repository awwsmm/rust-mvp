use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use device::id::Id;

use crate::assessor::Assessor;

pub struct State {
    pub(crate) sensors: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    pub(crate) actuators: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    pub(crate) assessors: Arc<Mutex<HashMap<Id, Assessor>>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sensors: Arc::new(Mutex::new(HashMap::new())),
            actuators: Arc::new(Mutex::new(HashMap::new())),
            assessors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
