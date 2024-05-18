//! Reference to a package

use crate::buckets;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Package reference errors
pub enum Error {
    #[error("Attempted to set bucket on a file path or url. This is not supported.")]
    BucketOnDirectRef,
    #[error("Invalid app name in manifest ref")]
    MissingAppName,
    #[error("IO Error")]
    Io(#[from] std::io::Error),
    #[error("Package name was not provided")]
    MissingPackageName,
    #[error(
        "Too many segments in package reference. Expected either `<bucket>/<name>` or `<name>`"
    )]
    TooManySegments,
    #[error("Invalid version supplied")]
    InvalidVersion,
    #[error("Could not find matching manifest")]
    NotFound,
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Packages error: {0}")]
    Packages(#[from] super::Error),
    #[error("ser/de error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Buckets error: {0}")]
    Buckets(#[from] buckets::Error),
}

pub mod manifest;
pub mod package;

mod ser_de {
    use std::str::FromStr;

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{manifest, package};

    impl Serialize for package::Reference {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for package::Reference {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            package::Reference::from_str(&s).map_err(serde::de::Error::custom)
        }
    }

    impl Serialize for manifest::Reference {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for manifest::Reference {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let s = String::deserialize(deserializer)?;
            manifest::Reference::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}
