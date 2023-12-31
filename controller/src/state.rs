use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use device::id::Id;
use device::Targets;

use crate::assessor::Assessor;

pub struct State {
    pub(crate) sensors: Targets,
    pub(crate) actuators: Targets,
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
