//! A nicer way to display keys

#![allow(deprecated)]

use std::fmt::Display;

#[derive(Debug, Clone)]
#[must_use]
#[deprecated(note = "Use `Header` instead")]
/// A nicer way to display keys
pub struct Key<T>(T);

impl<T> Key<T> {
    /// Wrap the provided key in a [`Key`]
    pub fn wrap(key: T) -> Self {
        Self(key)
    }
}

impl<T: Display> Display for Key<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_string = self.0.to_string();
        let nice_key = key_string.replace('_', " ");

        nice_key.fmt(f)
    }
}
