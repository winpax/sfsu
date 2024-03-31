use std::{fmt, path::PathBuf, str::FromStr};

use itertools::Itertools as _;
use url::Url;

use super::{CreateManifest, Manifest};
use crate::buckets::Bucket;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Attempted to set bucket on a file path or url. This is not supported.")]
    BucketOnDirectRef,
    #[error("Invalid app name in manifest ref")]
    MissingAppName,
    #[error("IO Error")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Package {
    manifest: ManifestRef,
    version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestRef {
    BucketNamePair { bucket: String, name: String },
    Name(String),
    File(PathBuf),
    Url(Url),
}

impl ManifestRef {
    #[must_use]
    pub fn into_package_ref(self) -> Package {
        Package {
            manifest: self,
            version: None,
        }
    }
}

impl Package {
    /// Update the bucket string in the package reference
    ///
    /// # Errors
    /// - If the package reference is a url. Setting the bucket on a url reference is not supported
    pub fn set_bucket(&mut self, bucket: String) -> Result<(), Error> {
        match &mut self.manifest {
            ManifestRef::BucketNamePair {
                bucket: old_bucket, ..
            } => *old_bucket = bucket,
            ManifestRef::Name(name) => {
                self.manifest = ManifestRef::BucketNamePair {
                    bucket,
                    name: name.clone(),
                }
            }
            _ => return Err(Error::BucketOnDirectRef),
        }

        Ok(())
    }

    #[must_use]
    /// Just get the bucket name
    pub fn bucket(&self) -> Option<&str> {
        match &self.manifest {
            ManifestRef::BucketNamePair { bucket, .. } => Some(bucket),
            _ => None,
        }
    }

    #[must_use]
    /// Get the package name
    pub fn name(&self) -> Option<String> {
        match &self.manifest {
            ManifestRef::Name(name) | ManifestRef::BucketNamePair { name, .. } => {
                Some(name.to_string())
            }
            ManifestRef::File(path) => {
                Some(path.with_extension("").file_name()?.to_str()?.to_string())
            }
            ManifestRef::Url(url) => {
                Some(url.path_segments()?.last()?.split('.').next()?.to_string())
            }
        }
    }

    #[must_use]
    /// Parse the bucket and package to get the manifest
    ///
    /// Returns [`None`] if the bucket is not valid, the manifest does not exist,
    /// or an error was thrown while getting the manifest
    pub fn manifest(&self) -> Option<Manifest> {
        // TODO: Map output to fix version

        if matches!(self.manifest, ManifestRef::File(_) | ManifestRef::Url(_)) {
            let mut manifest = match &self.manifest {
                ManifestRef::File(path) => Manifest::from_path(path).ok()?,
                ManifestRef::Url(url) => {
                    let manifest_string = crate::requests::BlockingClient::new()
                        .get(url.to_string())
                        .send()
                        .ok()?
                        .text()
                        .ok()?;

                    Manifest::from_str(manifest_string).ok()?
                }
                _ => unreachable!(),
            };

            manifest.name = self.name()?;

            return Some(manifest);
        }

        if let Some(bucket_name) = self.bucket() {
            let bucket = Bucket::from_name(bucket_name).ok()?;

            bucket.get_manifest(self.name()?).ok()
        } else {
            Bucket::list_all()
                .ok()?
                .into_iter()
                .find_map(|bucket| bucket.get_manifest(self.name()?).ok())
        }
    }

    #[must_use]
    /// Parse the bucket and package to get the manifest, or search for all matches in local buckets
    ///
    /// Returns a [`Vec`] with a single manifest if the reference is valid
    ///
    /// Otherwise returns a [`Vec`] containing each matching manifest found in each local bucket
    pub fn list_manifests(&self) -> Vec<Manifest> {
        if let Some(manifest) = self.manifest() {
            vec![manifest]
        } else {
            let Ok(buckets) = Bucket::list_all() else {
                return vec![];
            };

            buckets
                .into_iter()
                .filter_map(|bucket| match bucket.get_manifest(self.name()?) {
                    Ok(manifest) => Some(manifest),
                    Err(_) => None,
                })
                .collect()
        }
    }

    /// Checks if the package is installed
    ///
    /// # Errors
    /// - Reading app dir fails
    /// - Missing app name
    pub fn installed(&self) -> Result<bool, Error> {
        let name = self.name().ok_or(Error::MissingAppName)?;

        Ok(crate::Scoop::app_installed(name)?)
    }
}

impl From<ManifestRef> for Package {
    fn from(manifest: ManifestRef) -> Self {
        Package {
            manifest,
            version: None,
        }
    }
}

impl fmt::Display for ManifestRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ManifestRef::BucketNamePair { bucket, name } => write!(f, "{bucket}/{name}"),
            ManifestRef::Name(name) => write!(f, "{name}"),
            ManifestRef::File(_) => {
                let name = Package::from(self.clone()).name().unwrap();
                write!(f, "{name}")
            }
            ManifestRef::Url(url) => write!(f, "{url}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PackageRefParseError {
    #[error("Package name was not provided")]
    MissingPackageName,
    #[error(
        "Too many segments in package reference. Expected either `<bucket>/<name>` or `<name>`"
    )]
    TooManySegments,
    #[error("Invalid version supplied")]
    InvalidVersion,
}

impl FromStr for ManifestRef {
    type Err = PackageRefParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(url) = url::Url::parse(s) {
            return Ok(Self::Url(url));
        }

        if let Ok(path) = PathBuf::from_str(s) {
            if path.exists() {
                return Ok(Self::File(path));
            }
        }

        let parts = s.split('/').collect_vec();
        if parts.len() == 1 {
            Ok(Self::Name(parts[0].to_string()))
        } else if parts.len() == 2 {
            Ok(Self::BucketNamePair {
                bucket: parts[0].to_string(),
                name: parts[1].to_string(),
            })
        } else if parts.len() > 2 {
            Err(PackageRefParseError::TooManySegments)
        } else if parts.is_empty() {
            Err(PackageRefParseError::MissingPackageName)
        } else {
            unreachable!()
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.manifest)?;

        if let Some(version) = &self.version {
            write!(f, "@{version}")?;
        }

        Ok(())
    }
}

impl FromStr for Package {
    type Err = PackageRefParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('@').collect_vec();

        match parts.len() {
            1 => Ok(Package {
                manifest: ManifestRef::from_str(s)?,
                version: None,
            }),
            2 => Ok(Package {
                manifest: ManifestRef::from_str(parts[0])?,
                version: Some(parts[1].to_string()),
            }),
            _ => Err(PackageRefParseError::InvalidVersion),
        }
    }
}

mod ser_de {
    use super::{FromStr, ManifestRef, Package};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    impl Serialize for Package {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for Package {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            Package::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    impl Serialize for ManifestRef {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for ManifestRef {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            ManifestRef::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}
