use std::fmt::Display;

#[derive(Debug, Clone)]
#[must_use]
pub struct Key<T>(T);

impl<T> Key<T> {
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
