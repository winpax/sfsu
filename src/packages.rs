use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::get_scoop_path;

pub trait FromPath {
    /// Convert a path into a manifest
    ///
    /// # Errors
    /// - The file does not exist
    /// - The file was not valid UTF-8
    fn from_path(path: &Path) -> anyhow::Result<Self>
    where
        Self: for<'a> Deserialize<'a>,
    {
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(serde_json::from_str(
            contents.trim_start_matches('\u{feff}'),
        )?)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    /// The version of the package
    #[serde(default = "String::new")]
    pub version: String,
}

impl FromPath for Manifest {}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct InstallManifest {
    /// The bucket the package was installed from
    #[serde(default = "String::new")]
    pub bucket: String,
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

    if let Ok(mut file) = File::open(installed_path) {
        if let Some(bucket) = bucket {
            let mut buf = String::new();

            file.read_to_string(&mut buf).unwrap();

            let manifest: InstallManifest =
                serde_json::from_str(buf.trim_start_matches('\u{feff}')).unwrap();

            manifest.bucket == bucket.as_ref()
        } else {
            true
        }
    } else {
        false
    }
}
