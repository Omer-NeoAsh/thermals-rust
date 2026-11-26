use std::io;

use crate::{Sample, Sensor};

pub trait Backend {
    fn discover(&mut self) -> io::Result<Vec<Sensor>>;

    fn sample(&mut self) -> io::Result<Vec<Sample>>;
}
