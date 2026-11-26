#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeviceId {
    name: String,
}

impl DeviceId {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
