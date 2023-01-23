use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::get_scoop_path;

pub trait FromPath {
    /// Convert a path into a manifest
    ///
    /// # Errors
    /// - The file does not exist
    /// - The file was not valid UTF-8
    fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self>
    where
        Self: for<'a> Deserialize<'a>,
    {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(serde_json::from_str(contents.trim_start_matches('\u{feff}')).unwrap())
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default, deserialize_with = "ok_or_default")]
    /// The description of the package
    pub description: Option<String>,
    #[serde(default, deserialize_with = "ok_or_default")]
    /// The license of the package,
    pub license: Option<String>,
    #[serde(default, deserialize_with = "ok_or_default")]
    /// The homepage of the package
    pub homepage: Option<String>,
    #[serde(default = "default_version")]
    /// The version of the package
    pub version: String,
}

fn default_version() -> String {
    "Invalid Manifest".to_string()
}

fn ok_or_default<'a, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'a> + Default,
    D: serde::Deserializer<'a>,
{
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    Ok(T::deserialize(v).unwrap_or_default())
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
