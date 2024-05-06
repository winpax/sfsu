//! Status information

use itertools::Itertools as _;
use quork::traits::truthy::ContainsTruth;
use serde::Serialize;

use crate::contexts::ScoopContext;
use crate::{buckets::Bucket, packages::reference::ManifestRef, Scoop};

use crate::packages::{reference, Manifest, Result};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
/// The status info
pub struct Info {
    /// The name of the package
    pub name: String,
    /// The current version
    pub current: String,
    /// The available version
    pub available: String,
    /// The missing dependencies
    pub missing_dependencies: Vec<reference::Package>,
    /// Additional information
    pub info: Option<String>,
}

impl Info {
    /// Parse [`Info`] from a local manifest
    ///
    /// # Errors
    /// - If the local manifest is missing
    /// - If the install manifest is missing
    ///
    /// # Panics
    /// - Invalid package reference name
    pub fn from_manifests(local_manifest: &Manifest, bucket: &Bucket) -> Result<Self> {
        let failed = {
            let installed = Scoop::app_installed(&local_manifest.name)?;

            let app_path = Scoop::apps_path()
                .join(&local_manifest.name)
                .join("current");

            !app_path.exists() && installed
        };

        debug!("Local manifest name: {}", local_manifest.name);
        let remote_manifest = bucket.get_manifest(&local_manifest.name)?;

        let install_manifest = local_manifest.install_manifest()?;

        let held = install_manifest.hold.unwrap_or_default();

        let missing_dependencies = local_manifest
            .depends()
            .into_iter()
            .map(ManifestRef::into_package_ref)
            .filter(|reference| {
                debug!(
                    "Checking if {} is installed.",
                    reference.name().expect("valid name")
                );
                !reference.installed().contains_truth()
            })
            .collect_vec();

        let mut info = String::new();

        if failed {
            info += "Install failed";
        }
        if held {
            info += "Held package";
        }

        Ok(Info {
            name: remote_manifest.name.clone(),
            current: local_manifest.version.to_string(),
            available: remote_manifest.version.to_string(),
            missing_dependencies,
            info: (!info.is_empty()).then_some(info),
        })
    }
}
