//! A wrapper for nicely serializing values that implement the Display trait

use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Copy, Clone)]
/// Serialize a value using the Display trait
pub struct SerializeDisplay<T>(T);

impl<T: Display> Display for SerializeDisplay<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Display> Serialize for SerializeDisplay<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<T: Display> From<T> for SerializeDisplay<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
