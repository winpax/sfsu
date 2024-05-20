//! A package reference with an optional version

use std::{fmt, path::PathBuf, str::FromStr};

use itertools::Itertools;

use crate::{
    buckets::Bucket,
    config,
    contexts::ScoopContext,
    handles::{self, packages::PackageHandle},
    let_chain,
    packages::{CreateManifest, Manifest},
    requests::Client,
};

use super::{manifest, Error};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A package reference with an optional version
pub struct Reference {
    /// The manifest reference
    pub manifest: manifest::Reference,
    /// The manifest version
    pub version: Option<String>,
}

impl Reference {
    #[must_use]
    /// Convert the [`ManifestRef`] into a [`Package`] reference
    pub fn from_ref(manifest: manifest::Reference) -> Self {
        Self {
            manifest,
            version: None,
        }
    }

    /// Update the bucket string in the package reference
    ///
    /// # Errors
    /// - If the package reference is a url. Setting the bucket on a url reference is not supported
    pub fn set_bucket(&mut self, bucket: String) -> Result<(), Error> {
        match &mut self.manifest {
            manifest::Reference::BucketNamePair {
                bucket: old_bucket, ..
            } => *old_bucket = bucket,
            manifest::Reference::Name(name) => {
                self.manifest = manifest::Reference::BucketNamePair {
                    bucket,
                    name: name.clone(),
                }
            }
            _ => return Err(Error::BucketOnDirectRef),
        }

        Ok(())
    }

    /// Update the package version in the package reference
    pub fn set_version(&mut self, version: String) {
        self.version = Some(version);
    }

    #[must_use]
    /// Just get the bucket name
    pub fn bucket(&self) -> Option<&str> {
        match &self.manifest {
            manifest::Reference::BucketNamePair { bucket, .. } => Some(bucket),
            _ => None,
        }
    }

    #[must_use]
    /// Get the package name
    pub fn name(&self) -> Option<String> {
        match &self.manifest {
            manifest::Reference::Name(name) | manifest::Reference::BucketNamePair { name, .. } => {
                Some(name.to_string())
            }
            manifest::Reference::File(path) => {
                Some(path.with_extension("").file_name()?.to_str()?.to_string())
            }
            #[cfg(feature = "manifest-hashes")]
            manifest::Reference::Url(url) => {
                Some(url.path_segments()?.last()?.split('.').next()?.to_string())
            }
        }
    }

    #[must_use]
    /// Parse the bucket and package to get the manifest path
    ///
    /// Returns [`None`] if the bucket is not valid or the manifest does not exist
    pub fn manifest_path(&self, ctx: &impl ScoopContext<config::Scoop>) -> Option<PathBuf> {
        if let Some(bucket_name) = self.bucket() {
            let bucket = Bucket::from_name(ctx, bucket_name).ok()?;

            Some(bucket.get_manifest_path(self.name()?))
        } else {
            None
        }
    }

    /// Parse the bucket and package to get the manifest
    ///
    /// # Errors
    /// - If the manifest does not exist
    /// - If the manifest is invalid
    /// - If the manifest is not found
    /// - If the app name is missing
    /// - If the app dir cannot be read
    /// - If the bucket is not valid
    /// - If the bucket is not found
    pub async fn manifest(
        &self,
        ctx: &impl ScoopContext<config::Scoop>,
    ) -> Result<Manifest, Error> {
        // TODO: Map output to fix version

        let mut manifest = if {
            cfg_if::cfg_if! {
                if #[cfg(feature = "manifest-hashes")] {
                    matches!(self.manifest, manifest::Reference::File(_) | manifest::Reference::Url(_))
                } else {
                    matches!(self.manifest, manifest::Reference::File(_))
                }
            }
        } {
            let mut manifest = match &self.manifest {
                manifest::Reference::File(path) => Manifest::from_path(path)?,
                #[cfg(feature = "manifest-hashes")]
                manifest::Reference::Url(url) => {
                    let manifest_string = Client::asynchronous()
                        .get(url.to_string())
                        .send()
                        .await?
                        .text()
                        .await?;

                    Manifest::from_str(manifest_string)?
                }
                _ => unreachable!(),
            };

            manifest.name = self.name().ok_or(Error::MissingAppName)?;

            Ok(manifest)
        } else if let Some(bucket_name) = self.bucket() {
            let bucket = Bucket::from_name(ctx, bucket_name)?;

            Ok(bucket.get_manifest(self.name().ok_or(Error::MissingAppName)?)?)
        } else {
            Ok(Bucket::list_all(ctx)?
                .into_iter()
                .find_map(|bucket| bucket.get_manifest(self.name()?).ok())
                .ok_or(Error::NotFound)?)
        };

        #[cfg(feature = "manifest-hashes")]
        let_chain!(let Ok(manifest) = manifest.as_mut(); let Some(version) = &self.version; {
            manifest.set_version(ctx,version.clone()).await?;
        });

        manifest
    }

    #[must_use]
    /// Find the first matching manifest in local buckets
    ///
    /// Returns [`None`] if no matching manifest is found
    pub fn first(&self, ctx: &impl ScoopContext<config::Scoop>) -> Option<Manifest> {
        let Ok(buckets) = Bucket::list_all(ctx) else {
            return None;
        };

        buckets
            .into_iter()
            .find_map(|bucket| match bucket.get_manifest(self.name()?) {
                Ok(manifest) => Some(manifest),
                Err(_) => None,
            })
    }

    #[must_use]
    /// Parse the bucket and package to get the manifest path, or search for all matches in local buckets
    ///
    /// Returns a [`Vec`] with a single manifest path if the reference is valid
    ///
    /// Otherwise returns a [`Vec`] containing each matching manifest path found in each local bucket
    pub fn list_manifest_paths(&self, ctx: &impl ScoopContext<config::Scoop>) -> Vec<PathBuf> {
        if let Some(manifest_path) = self.manifest_path(ctx) {
            vec![manifest_path]
        } else {
            let Ok(buckets) = Bucket::list_all(ctx) else {
                return vec![];
            };

            buckets
                .into_iter()
                .filter_map(|bucket| {
                    let manifest_path = bucket.get_manifest_path(self.name()?);
                    if manifest_path.exists() {
                        Some(manifest_path)
                    } else {
                        None
                    }
                })
                .collect()
        }
    }

    /// Parse the bucket and package to get the manifest, or search for all matches in local buckets
    ///
    /// Returns a [`Vec`] with a single manifest if the reference is valid
    ///
    /// Otherwise returns a [`Vec`] containing each matching manifest found in each local bucket
    ///
    /// # Errors
    /// - If any of the manifests are invalid
    /// - If any of the manifests are not found
    /// - If any of the manifests are missing
    /// - If the app dir cannot be read
    /// - If any of the buckets are not valid
    /// - If any of the buckets are not found
    pub async fn list_manifests(
        &self,
        ctx: &impl ScoopContext<config::Scoop>,
    ) -> Result<Vec<Manifest>, Error> {
        futures::future::try_join_all(
            self.list_manifest_paths(ctx)
                .into_iter()
                .map(Manifest::from_path)
                .map(|manifest| async {
                    let mut manifest = manifest?;
                    #[cfg(feature = "manifest-hashes")]
                    if let Some(version) = &self.version {
                        manifest.set_version(ctx, version.clone()).await?;
                    }

                    Ok::<Manifest, Error>(manifest)
                }),
        )
        .await
    }

    /// Checks if the package is installed
    ///
    /// # Errors
    /// - Reading app dir fails
    /// - Missing app name
    pub fn installed(&self, ctx: &impl ScoopContext<config::Scoop>) -> Result<bool, Error> {
        let name = self.name().ok_or(Error::MissingAppName)?;

        Ok(ctx.app_installed(name)?)
    }

    /// Open a package handle
    ///
    /// # Errors
    /// - The package is not installed
    pub async fn open_handle<C: ScoopContext<config::Scoop>>(
        self,
        ctx: &C,
    ) -> Result<PackageHandle<'_, C>, handles::packages::Error> {
        PackageHandle::new(ctx, self).await
    }
}

impl From<manifest::Reference> for Reference {
    fn from(manifest: manifest::Reference) -> Self {
        Self::from_ref(manifest)
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.manifest)?;

        if let Some(version) = &self.version {
            write!(f, "@{version}")?;
        }

        Ok(())
    }
}

impl FromStr for Reference {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('@').collect_vec();

        match parts.len() {
            1 => Ok(Reference {
                manifest: manifest::Reference::from_str(s)?,
                version: None,
            }),
            2 => Ok(Reference {
                manifest: manifest::Reference::from_str(parts[0])?,
                version: Some(parts[1].to_string()),
            }),
            _ => Err(Error::InvalidVersion),
        }
    }
}
