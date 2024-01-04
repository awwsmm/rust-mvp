use phf::{phf_map, Map};

use datum::unit::Unit;
use datum::Datum;

#[derive(Clone)]
pub struct Assessor {
    pub(crate) assess: fn(&Datum) -> Option<Box<dyn actuator::Command>>,
}

/// Default `Assessor`s for different `Model`s of `Device`.
///
/// Can be overridden by the user.
pub static DEFAULT_ASSESSOR: Map<&str, Assessor> = phf_map! {
    // keys here should match Model ids defined in model.rs
    "thermo5000" => Assessor { assess: |datum| {

        let t = datum.get_as_float().unwrap();
        assert_eq!(datum.unit, Unit::DegreesC);

        if t > 25.0 {
            Some(Box::new(actuator_temperature::command::Command::CoolBy(t - 25.0)))
        } else if t < 25.0 {
            Some(Box::new(actuator_temperature::command::Command::HeatBy(25.0 - t)))
        } else {
            None
        }
    }}
};
