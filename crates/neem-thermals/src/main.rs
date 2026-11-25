use neem_thermals_core::{Celsius, Sample, SensorId, Value};

fn main() {
    for raw in [48.5, f32::NAN] {
        match Celsius::new(raw) {
            Some(temp) => {
                let sample = Sample {
                    sensor: SensorId::new("k10temp@0000:00:18.3/temp1"),
                    value: Value::Temperature(temp),
                };
                println!("{sample}");
            }
            None => eprintln!("rejected reading: {raw}"),
        }
    }
}
