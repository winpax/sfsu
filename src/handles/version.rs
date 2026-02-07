//! Provides version handles

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Version handle errors
pub enum Error {
    #[error("Missing version file name")]
    MissingFileName,
    #[error("Version path is current directory (this can be ignored)")]
    PathIsCurrent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A version handle
///
/// Provides a reference to a particular version's directory
pub struct VersionHandle {
    version: String,
    path: PathBuf,
}

impl VersionHandle {
    #[must_use]
    #[inline]
    /// Get the version string
    pub fn version(&self) -> &str {
        &self.version
    }

    #[must_use]
    #[inline]
    /// Get the version's directory
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Convert the version into a semver version requirement
    ///
    /// # Errors
    /// - The version could not be parsed as a semver version requirement
    pub fn to_semver(&self) -> Result<semver::Version, semver::Error> {
        semver::Version::parse(self.version())
    }
}

impl PartialOrd for VersionHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if let Ok(semver) = self.to_semver()
            && let Ok(other_semver) = other.to_semver()
        {
            semver.partial_cmp(&other_semver)
        } else {
            None
        }
    }
}

impl TryFrom<&Path> for VersionHandle {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let path = value.to_path_buf();
        let file_name = path.file_name().ok_or(Error::MissingFileName)?;

        if file_name == "current" {
            return Err(Error::PathIsCurrent);
        }

        let version = file_name.to_string_lossy().to_string();

        Ok(Self { version, path })
    }
}

impl TryFrom<PathBuf> for VersionHandle {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        TryFrom::<&Path>::try_from(&value)
    }
}
