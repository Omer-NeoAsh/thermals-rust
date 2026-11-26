use crate::{DeviceId, Kind, SensorId};

#[derive(Clone, Debug, PartialEq)]
pub struct Sensor {
    pub id: SensorId,
    pub label: String,
    pub device: DeviceId,
    pub kind: Kind,
}
