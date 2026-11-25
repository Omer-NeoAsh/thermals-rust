use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Celsius {
    degrees: f32,
}

impl Celsius {
    const MAX: f32 = 150.0;

    pub fn new(raw: f32) -> Option<Self> {
        (raw > 0.0 && raw <= Self::MAX).then_some(Self { degrees: raw })
    }

    pub fn degrees(self) -> f32 {
        self.degrees
    }
}

impl fmt::Display for Celsius {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.1} °C", self.degrees)
    }
}

#[cfg(test)]
mod tests;
