use phf::{phf_map, Map};

use datum::Datum;
use datum::DatumUnit;

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
        assert_eq!(datum.unit, DatumUnit::DegreesC);

        if t > 28.0 {
            Some(Box::new(actuator_temperature::command::Command::CoolTo(25.0)))
        } else if t < 26.0 {
            Some(Box::new(actuator_temperature::command::Command::HeatTo(25.0)))
        } else {
            None
        }
    }}
};
