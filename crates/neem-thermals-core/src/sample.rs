use std::fmt;

use crate::{SensorId, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub sensor: SensorId,
    pub value: Value,
}

impl fmt::Display for Sample {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.sensor, self.value)
    }
}

#[cfg(test)]
mod tests;
