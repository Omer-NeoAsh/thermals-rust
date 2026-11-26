use std::io;

use neem_thermals_core::{Backend, Celsius, DeviceId, Kind, Sample, Sensor, SensorId, Value};

const SENSOR_KEY: &str = "fake/cpu";
const COOLEST: f32 = 45.0;
const HOTTEST: f32 = 60.0;
const STEP: f32 = 0.5;

pub struct FakeBackend {
    degrees: f32,
}

impl FakeBackend {
    pub fn new() -> Self {
        Self { degrees: COOLEST }
    }
}

impl Backend for FakeBackend {
    fn discover(&mut self) -> io::Result<Vec<Sensor>> {
        Ok(vec![Sensor {
            id: SensorId::new(SENSOR_KEY),
            label: "Fake CPU".to_string(),
            device: DeviceId::new("cpu"),
            kind: Kind::Temperature,
        }])
    }

    fn sample(&mut self) -> io::Result<Vec<Sample>> {
        let reading = Celsius::new(self.degrees);
        self.degrees = if self.degrees >= HOTTEST {
            COOLEST
        } else {
            self.degrees + STEP
        };

        Ok(reading
            .map(|celsius| Sample {
                sensor: SensorId::new(SENSOR_KEY),
                value: Value::Temperature(celsius),
            })
            .into_iter()
            .collect())
    }
}
