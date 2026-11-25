use crate::{Celsius, Sample, SensorId, Value};

#[test]
fn displays_sensor_key_and_value() {
    let sample = Sample {
        sensor: SensorId::new("k10temp@0000:00:18.3/temp1"),
        value: Value::Temperature(Celsius::new(48.5).unwrap()),
    };

    assert_eq!(sample.to_string(), "k10temp@0000:00:18.3/temp1: 48.5 °C")
}
