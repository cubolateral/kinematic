/// User-facing name attached to every scene object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    /// Creates an object name.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the current object name.
    pub fn get(&self) -> &str {
        &self.0
    }

    /// Replaces the current object name.
    pub fn set(&mut self, value: impl Into<String>) {
        self.0 = value.into();
    }
}
