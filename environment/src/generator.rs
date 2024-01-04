use chrono::{DateTime, Utc};
use rand::random;

use datum::unit::Unit;
use datum::Datum;

// y = a + b*x + c*sin(d(x+e))
pub struct Coefficients {
    pub constant: f32, // a
    pub slope: f32,    // b
    amplitude: f32,    // c
    period: f32,       // 2*pi / d
    phase: f32,        // e
}

impl Coefficients {
    pub fn new(constant: f32, slope: f32, amplitude: f32, period: f32, phase: f32) -> Coefficients {
        Coefficients {
            constant,
            slope,
            amplitude,
            period,
            phase,
        }
    }

    pub fn random() -> Coefficients {
        let constant = random::<f32>() - 0.5;
        let slope = random::<f32>() - 0.5;
        let amplitude = random::<f32>() - 0.5;
        let period = random::<f32>() - 0.5;
        let phase = random::<f32>() - 0.5;
        Coefficients {
            constant,
            slope,
            amplitude,
            period,
            phase,
        }
    }
}

pub struct DatumGenerator {
    t0: DateTime<Utc>,
    pub coefficients: Coefficients,
    noise: f32,
    unit: Unit,
}

impl DatumGenerator {
    pub fn new(coefficients: Coefficients, noise: f32, unit: Unit) -> DatumGenerator {
        DatumGenerator {
            t0: Utc::now(),
            coefficients,
            noise,
            unit,
        }
    }

    #[allow(dead_code)] // FIXME remove ASAP
    pub fn random(unit: Unit) -> DatumGenerator {
        DatumGenerator {
            t0: Utc::now(),
            coefficients: Coefficients::random(),
            noise: random::<f32>() - 0.5,
            unit,
        }
    }

    pub fn generate(&self) -> Datum {
        let now = Utc::now();

        // converting i64 to f32 is safe as long as this demo is running for < 9.4e28 hours
        let x = (now - self.t0).num_milliseconds() as f32;
        let Coefficients {
            constant,
            slope,
            amplitude,
            period,
            phase,
        } = self.coefficients;

        let noise = (random::<f32>() - 0.5) * self.noise;
        let value = constant + slope * x + amplitude * f32::sin((2.0 * std::f32::consts::PI / period) * (x + phase)) + noise;

        Datum::new(value, self.unit, now)
    }
}

#[cfg(test)]
mod generator_tests {
    use std::thread::sleep;

    use chrono::Duration;

    use super::*;

    #[test]
    fn test_constant() {
        let coefficients = Coefficients::new(5.0, 0.0, 0.0, 1.0, 0.0);
        let noise = 0.0;
        let generator = DatumGenerator::new(coefficients, noise, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is equal to a value generated later
        assert_eq!(earlier.get_as_float(), later.get_as_float());

        // all values should be equal to the provided constant, 5.0
        assert_eq!(earlier.get_as_float(), Some(5.0));
    }

    #[test]
    /// Slope is positive -- tests that a value generated earlier is less than a value generated later
    fn test_linear_positive_slope() {
        let coefficients = Coefficients::new(0.0, 1.0, 0.0, 1.0, 0.0);
        let noise = 0.0;
        let generator = DatumGenerator::new(coefficients, noise, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is less than a value generated later
        assert!(earlier.get_as_float() < later.get_as_float());
    }

    #[test]
    /// Slope is negative -- tests that a value generated earlier is greater than a value generated later
    fn test_linear_negative_slope() {
        let coefficients = Coefficients::new(0.0, -1.0, 0.0, 1.0, 0.0);
        let noise = 0.0;
        let generator = DatumGenerator::new(coefficients, noise, Unit::DegreesC);

        // generate a datum, wait, then generate another
        let earlier = generator.generate();
        sleep(Duration::milliseconds(1).to_std().unwrap());
        let later = generator.generate();

        // a value generated earlier is greater than a value generated later
        assert!(earlier.get_as_float() > later.get_as_float());
    }
}
