use std::fmt::Display;

use super::consts::{SUFFIX, WALL};

pub struct TruncateOrPad<T>(T, usize);

impl<T> TruncateOrPad<T> {
    pub fn new(data: T, length: usize) -> Self {
        Self(data, length)
    }
}

impl<T: Display> Display for TruncateOrPad<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let length = self.1 - WALL.len();
        let data = self.0.to_string();
        if data.len() > length {
            write!(
                f,
                "{}{SUFFIX}",
                &data[0..length.checked_sub(3).unwrap_or_default()]
            )
        } else {
            write!(f, "{:width$}", self.0, width = length)
        }
    }
}
