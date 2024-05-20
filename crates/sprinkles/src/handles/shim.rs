//! Shim handles

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Shim errors
pub enum Error {
    #[error("Deleting shims: {0}")]
    IOError(#[from] std::io::Error),
}

/// Shim result type
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Flags for deleting shims
pub struct DeleteFlags(u8);

impl DeleteFlags {
    /// Delete just the executable
    pub const EXECUTABLE: Self = Self(0b10);
    /// Delete just the shim
    pub const SHIM: Self = Self(0b01);

    #[must_use]
    /// Check if we should delete the executable
    pub fn is_executable(self) -> bool {
        self & Self::EXECUTABLE == Self::EXECUTABLE
    }

    #[must_use]
    /// Check if we should delete the shim
    pub fn is_shim(self) -> bool {
        self & Self::SHIM == Self::SHIM
    }
}

impl std::ops::BitOr for DeleteFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for DeleteFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

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

    /// Delete the shim and executable
    ///
    /// # Errors
    /// - Deleting the shim failed
    /// - Deleting the executable failed
    pub fn delete_all(&self) -> Result<()> {
        self.delete(DeleteFlags::EXECUTABLE | DeleteFlags::SHIM)
    }

    /// Delete the shim and/or the executable
    ///
    /// # Examples
    /// ```no_run
    /// # use sprinkles::packages::handles::{ShimHandle, DeleteFlags};
    /// # let shim = ShimHandle::new(PathBuf::from("executable.exe"), PathBuf::from("shim.exe"));
    /// shim.delete(DeleteFlags::EXECUTABLE | DeleteFlags::SHIM);
    /// ```
    ///
    /// # Errors
    /// - Deleting the shim failed
    /// - Deleting the executable failed
    pub fn delete(&self, flags: DeleteFlags) -> Result<()> {
        if flags.is_executable() {
            if let Some(executable) = self.executable() {
                std::fs::remove_file(executable)?;
            }
        }

        if flags.is_shim() {
            if let Some(shim) = self.shim() {
                std::fs::remove_file(shim)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_flags() {
        assert_eq!(DeleteFlags::EXECUTABLE.0, 0b10);
        assert_eq!(DeleteFlags::SHIM.0, 0b01);
        assert_eq!((DeleteFlags::EXECUTABLE | DeleteFlags::SHIM).0, 0b11);

        let flags = DeleteFlags::EXECUTABLE | DeleteFlags::SHIM;

        assert!(flags.is_executable());
        assert!(flags.is_shim());

        let flags = DeleteFlags::EXECUTABLE;

        assert!(flags.is_executable());
        assert!(!flags.is_shim());

        let flags = DeleteFlags::SHIM;

        assert!(!flags.is_executable());
        assert!(flags.is_shim());
    }
}
