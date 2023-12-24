use std::time::Duration;

use device::{Device, Id, Model, Name};

use crate::state::State;

mod state;

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so some historic data is held
/// in memory.
pub struct Controller {
    name: Name,
    model: Model,
    id: Id,
    state: State,
}

impl Device for Controller {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_model(&self) -> &Model {
        &self.model
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            name: Name::new("controller"),
            model: Model::new("controller"),
            id: Id::new("controller"),
            state: State::new(),
        }
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller::default()
    }

    /// Starts the discovery process as well as polling sensors
    pub fn run(&mut self) {
        // spawn a thread to look for sensors on the network continually
        self.state.discover_sensors();

        // spawn a thread to look for actuators on the network continually
        self.state.discover_actuators();

        // poll sensors for data in perpetuity, waiting 1s in between polls
        self.state.poll();

        // TODO I think we need two more loops
        //      Loop 1 should be another state-internal loop, polling all known sensors for data and saving it in the histories
        //      Loop 2 should be in this scope right here, and it should be the "control loop".
        //
        //      The "control loop" should
        //        1. get the latest Datums for each sensor
        //        2. determine if each sensor is outside of some user-defined range
        //        3. if so, command the sensor's corresponding actuator to perform some command

        // // Cycle through and poll the Sensors, if the return Datum is outside a defined range
        // // send a command off to the Actuator
        // let self_api_clone = Arc::clone(&self.state);
        //
        // // FIXME I think this loop below needs to happen inside the State, like the discover() loop
        // std::thread::spawn(move || loop {
        //
        //     // acquire a mutex lock on the state
        //     let mut ctrl = self_api_clone.lock().unwrap();
        //
        //     // loop over all known sensors
        //     for (id, )
        //
        //
        //
        //     // Create a temp vec to hold the data history as there is a lock on the controller and
        //     // we can't populate the history until the lock is released.
        //     let mut data_history: Vec<(Id, SensorHistory)> = Vec::new();
        //     {
        //         let ctrl = self_api_clone.lock().unwrap();
        //
        //     }
        //
        //     // Once we have exited the scope where we acquired the data and send commands
        //     // its safe to acquire lock on ctrl again and update its data history
        //     let mut ctrl = self_api_clone.lock().unwrap();
        //     for (id, history) in data_history {
        //         ctrl.data.insert(id, history);
        //     }
        //     std::thread::sleep(Duration::from_secs(5));
        // });
        //
        // Ok(())

        // run() should loop continually
        std::thread::sleep(Duration::MAX)
    }
}
