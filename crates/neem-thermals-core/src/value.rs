use std::fmt;

use crate::Celsius;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Temperature(Celsius),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Temperature(celsius) => celsius.fmt(formatter),
        }
    }
}
