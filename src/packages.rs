use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::get_scoop_path;

pub trait FromPath
where
    Self: Default,
{
    /// Convert a path into a manifest
    ///
    /// # Errors
    /// - The file does not exist
    /// - The file was not valid UTF-8
    fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self>
    where
        Self: for<'a> Deserialize<'a>,
    {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(serde_json::from_str(contents.trim_start_matches('\u{feff}')).unwrap_or_default())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    /// The version of the package
    pub version: String,
}

impl Default for Manifest {
    fn default() -> Self {
        Manifest {
            version: "Invalid".to_string(),
        }
    }
}

impl FromPath for Manifest {}

#[derive(Debug, Deserialize, Serialize)]
pub struct InstallManifest {
    /// The bucket the package was installed from
    pub bucket: String,
}

impl Default for InstallManifest {
    fn default() -> Self {
        InstallManifest {
            bucket: "Invalid".to_string(),
        }
    }
}

impl FromPath for InstallManifest {}

/// Check if the manifest path is installed, and optionally confirm the bucket
///
/// # Panics
/// - The file was not valid UTF-8
pub fn is_installed(manifest_name: impl AsRef<Path>, bucket: Option<impl AsRef<str>>) -> bool {
    let scoop_path = get_scoop_path();
    let installed_path = scoop_path
        .join("apps")
        .join(manifest_name)
        .join("current/install.json");

    match InstallManifest::from_path(installed_path) {
        Ok(manifest) => {
            if let Some(bucket) = bucket {
                manifest.bucket == bucket.as_ref()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
