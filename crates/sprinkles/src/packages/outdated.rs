//! Outdated package information

use serde::Serialize;

use super::Manifest;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
/// The outdated info
pub struct Info {
    /// The name of the package
    pub name: String,
    /// The current version
    pub current: String,
    /// The available version
    pub available: String,
}

impl Info {
    #[must_use]
    /// Get the outdated info from a local and remote manifest combo
    ///
    /// Returns [`None`] if they have the same version
    pub fn from_manifests(local: &Manifest, remote: &Manifest) -> Option<Self> {
        if local.version == remote.version {
            None
        } else {
            Some(Info {
                name: remote.name.clone(),
                current: local.version.to_string(),
                available: remote.version.to_string(),
            })
        }
    }
}
