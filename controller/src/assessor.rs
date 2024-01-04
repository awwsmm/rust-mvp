use phf::{phf_map, Map};

use actuator_temperature::command::Command as Thermo5000;
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

        if t > 28.0 {
            Some(Box::new(Thermo5000::CoolBy(t - 25.0)))
        } else if t < 22.0 {
            Some(Box::new(Thermo5000::HeatBy(25.0 - t)))
        } else {
            None
        }
    }}
};

#[cfg(test)]
mod assessor_tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_thermo5000() {
        let assessor = DEFAULT_ASSESSOR.get("thermo5000").unwrap();

        let too_cold = Datum::new(21.0, Unit::DegreesC, Utc::now());

        let actual = (assessor.assess)(&too_cold).unwrap();
        let expected = Thermo5000::HeatBy(4.0);

        // it is very difficult to compare a Box<dyn actuator::Command> to a Thermo5000::Command
        // in lieu of directly comparing them, compare their serialized forms

        assert_eq!(actual.to_string(), expected.to_string());

        let too_hot = Datum::new(30.0, Unit::DegreesC, Utc::now());
        let actual = (assessor.assess)(&too_hot).unwrap();
        let expected = Thermo5000::CoolBy(5.0);

        assert_eq!(actual.to_string(), expected.to_string());

        let just_right = Datum::new(25.0, Unit::DegreesC, Utc::now());
        let actual = (assessor.assess)(&just_right);

        assert!(actual.is_none());
    }
}
