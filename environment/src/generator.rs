use std::ops::Add;

use chrono::{DateTime, Utc};

use datum::unit::Unit;
use datum::value::Value;
use datum::Datum;

pub struct DatumGenerator {
    generator: Box<dyn FnMut(DateTime<Utc>) -> Value>,
    pub(crate) unit: Unit,
}

unsafe impl Send for DatumGenerator {}

unsafe impl Sync for DatumGenerator {}

impl DatumGenerator {
    pub(crate) fn new(
        generator: Box<dyn FnMut(DateTime<Utc>) -> Value>,
        unit: Unit,
    ) -> DatumGenerator {
        DatumGenerator { generator, unit }
    }

    pub(crate) fn generate(&mut self) -> Datum {
        let now = Utc::now();
        let generator = &mut self.generator;
        let value = (*generator)(now);
        Datum::new(value, self.unit, now)
    }
}

impl Add for DatumGenerator {
    type Output = DatumGenerator;

    fn add(self, mut rhs: Self) -> Self::Output {
        let mut lhs = self;

        let unit = lhs.unit;

        let composed_generator = move |t: DateTime<Utc>| -> Value {
            let a = (*lhs.generator)(t);
            let b = (*rhs.generator)(t);

            match (a, b) {
                (Value::Float(a), Value::Float(b)) => Value::Float(a + b),
                (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                _ => panic!("[DatumGenerator] error during add operation"),
            }
        };

        Self::new(Box::new(composed_generator), unit)
    }
}

pub mod time_dependent {
    use chrono::{DateTime, Utc};
    use rand::{thread_rng, Rng};

    use datum::unit::Unit;
    use datum::value::Value;

    use crate::generator::DatumGenerator;

    pub fn f32_linear(slope: f32, noise: f32, unit: Unit) -> DatumGenerator {
        let start = Utc::now().timestamp_millis();
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> Value {
            // converting i64 to f32 is safe as long as this demo is running for < 9.4e28 hours
            let delta = (now.timestamp_millis() - start) as f32;
            let noise_factor = rng.gen_range(-1.0..1.0) * noise;
            Value::Float(delta * slope + noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }

    pub fn i32_linear(slope: i32, noise: i32, unit: Unit) -> DatumGenerator {
        let start = Utc::now().timestamp_millis();
        let mut rng = thread_rng();

        let f = move |now: DateTime<Utc>| -> Value {
            // truncating i64 to i32 is safe as long as this demo is running for < 596.5 hours
            let delta = (now.timestamp_millis() - start) as i32;
            let noise_factor = rng.gen_range(-1..1) * noise;
            Value::Int(delta * slope + noise_factor)
        };

        DatumGenerator::new(Box::new(f), unit)
    }
}

pub fn bool_alternating(initial: bool, unit: Unit) -> DatumGenerator {
    let mut latest_value = !initial;

    let f = move |_| -> Value {
        latest_value = !latest_value;
        Value::Bool(latest_value)
    };

    DatumGenerator::new(Box::new(f), unit)
}

#[cfg(test)]
mod generator_tests {
    use std::thread::sleep;

    use chrono::Duration;

    use super::*;

    #[test]
    /// Slope is positive -- tests that a value generated earlier is less than a value generated later
    fn test_f32_linear_positive_slope() {
        let slope = 1.0;
        let mut generator = time_dependent::f32_linear(slope, 0.0, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is less than a value generated later
        assert!(earlier.get_as_float() < later.get_as_float());
    }

    #[test]
    /// Slope is negative -- tests that a value generated earlier is greater than a value generated later
    fn test_f32_linear_negative_slope() {
        let slope = -1.0;
        let mut generator = time_dependent::f32_linear(slope, 0.0, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is greater than a value generated later
        assert!(earlier.get_as_float() > later.get_as_float());
    }

    #[test]
    /// Slope is positive -- tests that a value generated earlier is less than a value generated later
    fn test_i32_linear_positive_slope() {
        let slope = 1;
        let mut generator = time_dependent::i32_linear(slope, 0, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is less than a value generated later
        assert!(earlier.get_as_int() < later.get_as_int());
    }

    #[test]
    /// Slope is negative -- tests that a value generated earlier is greater than a value generated later
    fn test_i32_linear_negative_slope() {
        let slope = -1;
        let mut generator = time_dependent::i32_linear(slope, 0, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is greater than a value generated later
        assert!(earlier.get_as_int() > later.get_as_int());
    }

    #[test]
    fn test_bool_alternating() {
        let initial = false;
        let mut generator = bool_alternating(initial, Unit::DegreesC);

        // generate a datum, then generate another, and another
        let first = generator.generate();
        let second = generator.generate();
        let third = generator.generate();

        // values should start false (initial), then flip back and forth true to false, etc.
        assert_eq!(first.get_as_bool(), Some(false));
        assert_eq!(second.get_as_bool(), Some(true));
        assert_eq!(third.get_as_bool(), Some(false));
    }
}
