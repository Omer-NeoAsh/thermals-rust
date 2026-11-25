use std::fmt;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SensorId {
    key: String,
}

impl SensorId {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into() }
    }
}

impl fmt::Display for SensorId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.key)
    }
}
