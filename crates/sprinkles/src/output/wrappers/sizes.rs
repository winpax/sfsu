//! A wrapper for sizes in bytes.

use std::cmp::min;
use std::fmt::Display;

use serde::Serialize;

const SUFFIX: [&str; 9] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
const UNIT: f64 = 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
/// A size in bytes.
pub struct Size(f64);

impl Size {
    #[must_use]
    /// Create a new size.
    pub fn new(size: f64) -> Self {
        Self(size)
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 <= 0.0 {
            return write!(f, "0 B");
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let log = (self.0.ln() / UNIT.ln()).floor() as usize;
        let i = min(log, SUFFIX.len() - 1);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let size = self.0 / UNIT.powi(i as i32);

        write!(f, "{:.2} {}", size, SUFFIX[i])
    }
}

impl Serialize for Size {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(Size(0.0).to_string(), "0 B");
        assert_eq!(Size(1024.0).to_string(), "1.00 KiB");
        assert_eq!(Size(1024.0 * 1024.0).to_string(), "1.00 MiB");
        assert_eq!(Size(1024.0 * 1024.0 * 1024.0).to_string(), "1.00 GiB");
        assert_eq!(
            Size(1024.0 * 1024.0 * 1024.0 * 1024.0).to_string(),
            "1.00 TiB"
        );
        assert_eq!(
            Size(1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0).to_string(),
            "1.00 PiB"
        );
        assert_eq!(
            Size(1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0).to_string(),
            "1.00 EiB"
        );
        assert_eq!(
            Size(1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0).to_string(),
            "1.00 ZiB"
        );
        assert_eq!(
            Size(1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0).to_string(),
            "1.00 YiB"
        );
    }
}
