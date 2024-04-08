use serde::Serialize;

use super::Manifest;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct Info {
    pub name: String,
    pub current: String,
    pub available: String,
}

impl Info {
    /// Get the outdated info from a local and remote manifest combo
    ///
    /// Returns [`None`] if they have the same version
    #[must_use]
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
