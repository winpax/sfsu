use std::{fmt, str::FromStr};

use itertools::Itertools as _;

use super::Manifest;
use crate::buckets::Bucket;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Package {
    BucketNamePair { bucket: String, name: String },
    Name(String),
}

impl Package {
    /// Update the bucket string in the package reference
    pub fn set_bucket(&mut self, bucket: String) {
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
        }
    }

    #[must_use]
    /// Just get the bucket name
    pub fn bucket(&self) -> Option<&str> {
        match self {
            Package::BucketNamePair { bucket, .. } => Some(bucket),
            Package::Name(_) => None,
        }
    }

    #[must_use]
    /// Just get the package name
    pub fn name(&self) -> &str {
        match self {
            Package::Name(name) | Package::BucketNamePair { name, .. } => name,
        }
    }

    #[must_use]
    /// Parse the bucket and package to get the manifest
    ///
    /// Returns [`None`] if the bucket is not valid or the manifest does not exist
    pub fn manifest(&self) -> Option<Manifest> {
        if let Some(bucket_name) = self.bucket() {
            let bucket = Bucket::from_name(bucket_name).ok()?;

            bucket.get_manifest(self.name()).ok()
        } else {
            None
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
                .filter_map(|bucket| match bucket.get_manifest(self.name()) {
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
