//! A reference to a manifest

use std::{fmt, path::PathBuf, str::FromStr};

use itertools::Itertools;
use url::Url;

use super::{package, Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A reference to a package
pub enum Reference {
    /// Manifest reference with a bucket and name
    BucketNamePair {
        /// The package bucket
        bucket: String,
        /// The package name
        name: String,
    },
    /// Manifest reference with just a name
    Name(String),
    /// Manifest reference from path
    File(PathBuf),
    #[cfg(feature = "manifest-hashes")]
    /// Manifest reference from url
    Url(Url),
}

impl Reference {
    #[must_use]
    /// Convert the [`ManifestRef`] into a [`Package`] reference
    pub fn into_package_ref(self) -> package::Reference {
        package::Reference {
            manifest: self,
            version: None,
        }
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reference::BucketNamePair { bucket, name } => write!(f, "{bucket}/{name}"),
            Reference::Name(name) => write!(f, "{name}"),
            Reference::File(_) => {
                let name = package::Reference::from(self.clone()).name().unwrap();
                write!(f, "{name}")
            }
            #[cfg(feature = "manifest-hashes")]
            Reference::Url(url) => write!(f, "{url}"),
        }
    }
}

impl FromStr for Reference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        #[cfg(feature = "manifest-hashes")]
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
            Err(Error::TooManySegments)
        } else if parts.is_empty() {
            Err(Error::MissingPackageName)
        } else {
            unreachable!()
        }
    }
}
