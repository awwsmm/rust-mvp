# rust-mvp

This Rust IoT MVP was written over just 5 days by a team of 3-4 developers.

## crates

This Cargo workspace project contains a few crates. Each crate has a simple name which aligns with the [ubiquitous language](https://martinfowler.com/bliki/UbiquitousLanguage.html) used throughout this repository.

### sensor

This is a library crate (no `main.rs` file) which defines the basic library interface and the communication layer for an IoT _sensor_ running as a standalone device (at its own IP address). It is possible in this demo to define multiple sensors.

A sensor gathers information from the [environment](#environment) and provides it to the [controller](#controller) when requested.

Concrete (demo) implementations of sensors are held in directories with names starting with `sensor_`. Those crates are binary crates which can be containerized and run on a container runtime like Docker.

### actuator

This is another library crate (no `main.rs` file) which defines the basic library interface and the communication layer for an IoT _actuator_ running as a standalone device (at its own IP address). It is possible in this demo to define multiple actuators. In this demo, each sensor is paired with exactly one actuator.

An actuator receives commands from the [controller](#controller) and has an effect on the [environment](#environment).

Concrete (demo) implementations of actuators are held in directories with names starting with `actuator_`. Those crates are binary crates which can be containerized and run on a container runtime like Docker.

### controller

This is a binary crate which provides a concrete implementation of a _controller_, including all of its domain and communication logic. In this demo, there is only a single controller, which acts as the "hub" in this IoT system.

The controller collects data from one or more sensors, analyzes that data, and sends commands to one or more actuators.

In this demo, we use [mDNS](https://en.wikipedia.org/wiki/Multicast_DNS) to connect the controller to the sensors and actuators; they are automatically detected as they join the network. We also use a _pull_ mechanism wherein the controller queries the sensors for data (rather than the sensors _pushing_ data to the controller) ; this allows for backpressure and ensures the controller is never overwhelmed by requests or data.

The controller crate can be containerized and run on a container runtime like Docker.

### environment

This is a demo-only binary crate which acts as a mock environment for our IoT system. It contains information about the current state of the system, which may include mock temperature, humidity, and lighting data, among others.

The environment is mutated by the actuators and is probed by the sensors. In our demo, this occurs via communication over the network, like all other point-to-point communication.

The environment crate can be containerized and run on a container runtime like Docker.

### datum

_Datum_ is the singular form of _data_; a datum describes a single observation / measurement of some aspect of the environment. In our implementation, every datum has a value, an associated unit, and a timestamp.

### demo

This is the entrypoint to the demo. It contains a `main.rs` file which can be run locally to spin up our example IoT system and observe its behaviour.