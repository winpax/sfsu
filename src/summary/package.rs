use std::{path::Path, time::UNIX_EPOCH};

use chrono::NaiveDateTime;
use quork::traits::truthy::ContainsTruth as _;
use serde::{Deserialize, Serialize};

use crate::packages::{CreateManifest as _, InstallManifest, Manifest};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Summary {
    pub name: String,
    pub version: String,
    pub source: String,
    pub updated: String,
    pub notes: String,
}

impl Summary {
    /// Summarize a package from the provided path
    ///
    /// # Panics
    /// - Invalid or out-of-range Date Time
    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();

        let package_name = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .ok_or(anyhow::anyhow!("Missing or invalid file name"))?;

        let naive_time = {
            let updated = {
                let updated_sys = path.metadata()?.modified()?;

                updated_sys.duration_since(UNIX_EPOCH)?.as_secs()
            };

            NaiveDateTime::from_timestamp_opt(updated.try_into()?, 0)
                .expect("invalid or out-of-range datetime")
        };

        let app_current = path.join("current");

        let manifest = Manifest::from_path(app_current.join("manifest.json")).unwrap_or_default();

        let install_manifest =
            InstallManifest::from_path(app_current.join("install.json")).unwrap_or_default();

        Ok(Self {
            name: package_name.to_string(),
            version: manifest.version,
            source: install_manifest.get_source(),
            updated: naive_time.to_string(),
            notes: if install_manifest.hold.contains_truth() {
                String::from("Held")
            } else {
                String::new()
            },
        })
    }
}
