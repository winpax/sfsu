//! Shim handles

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
/// A shim handle
pub struct ShimHandle {
    executable: PathBuf,
    shim: PathBuf,
}

impl ShimHandle {
    #[must_use]
    /// Create a new shim handle
    pub fn new(executable: PathBuf, shim: PathBuf) -> Self {
        Self { executable, shim }
    }

    #[must_use]
    /// Get the executable path
    ///
    /// This will return the executable path if it exists, or [`None`] if it does not
    pub fn executable(&self) -> Option<&Path> {
        if self.executable.exists() {
            Some(self.executable.as_path())
        } else {
            None
        }
    }

    #[must_use]
    /// Get the shim path
    ///
    /// This will return the shim path if it exists, or [`None`] if it does not
    pub fn shim(&self) -> Option<&Path> {
        if self.shim.exists() {
            Some(&self.shim)
        } else {
            None
        }
    }
}
