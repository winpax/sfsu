use std::{fmt, path::PathBuf, str::FromStr};

use itertools::Itertools as _;
use url::Url;

use super::{CreateManifest, Manifest};
use crate::buckets::Bucket;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Attempted to set bucket on a file path or url. This is not supported.")]
    BucketOnDirectRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Package {
    BucketNamePair { bucket: String, name: String },
    Name(String),
    File(PathBuf),
    Url(Url),
}

impl Package {
    /// Update the bucket string in the package reference
    ///
    /// # Errors
    /// - If the package reference is a url. Setting the bucket on a url reference is not supported
    pub fn set_bucket(&mut self, bucket: String) -> Result<(), Error> {
        match self {
            Package::BucketNamePair {
                bucket: old_bucket, ..
            } => *old_bucket = bucket,
            Package::Name(name) => {
                *self = Package::BucketNamePair {
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
        match self {
            Package::BucketNamePair { bucket, .. } => Some(bucket),
            _ => None,
        }
    }

    #[must_use]
    /// Just get the package name
    ///
    /// Returns [`None`] if the reference was a url
    pub fn name(&self) -> Option<String> {
        match self {
            Package::Name(name) | Package::BucketNamePair { name, .. } => Some(name.to_string()),
            Package::File(path) => Some(path.with_extension("").file_name()?.to_str()?.to_string()),
            Package::Url(_) => None,
        }
    }

    #[must_use]
    /// Parse the bucket and package to get the manifest
    ///
    /// Returns [`None`] if the bucket is not valid, the manifest does not exist,
    /// or an error was thrown while getting the manifest
    pub fn manifest(&self) -> Option<Manifest> {
        if matches!(self, Self::File(_) | Self::Url(_)) {
            let manifest = match self {
                Package::File(path) => Manifest::from_path(path).ok()?,
                Package::Url(url) => {
                    let manifest_string =
                        reqwest::blocking::get(url.to_string()).ok()?.text().ok()?;

                    Manifest::from_str(manifest_string).ok()?
                }
                _ => unreachable!(),
            };

            return Some(manifest);
        }

        if let Some(bucket_name) = self.bucket() {
            let bucket = Bucket::new(bucket_name).ok()?;

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
    pub fn search_manifest(&self) -> Vec<Manifest> {
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
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Package::BucketNamePair { bucket, name } => write!(f, "{bucket}/{name}"),
            Package::Name(name) => write!(f, "{name}"),
            Package::File(_) => {
                let name = self.name().unwrap();
                write!(f, "{name}")
            }
            Package::Url(url) => write!(f, "{url}"),
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
}

impl FromStr for Package {
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

mod ser_de {
    use super::{FromStr, Package};
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
}
