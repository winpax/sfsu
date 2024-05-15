//! Scoop bucket helpers

use std::{
    borrow::Cow,
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use regex::Regex;

pub(crate) mod known {
    #![allow(clippy::unreadable_literal)]
    include!(concat!(env!("OUT_DIR"), "/buckets.rs"));
}

use crate::{
    config,
    contexts::ScoopContext,
    git::{self, Repo},
    packages::{self, CreateManifest, InstallManifest, Manifest, SearchMode},
};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Bucket errors
pub enum Error {
    #[error("Interacting with repo: {0}")]
    RepoError(#[from] git::Error),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("The bucket \"{0}\" does not exist")]
    InvalidBucket(PathBuf),
    #[error("Missing or invalid git output")]
    MissingGitOutput,
    #[error("Could not find executable in path: {0}")]
    MissingInPath(#[from] which::Error),
    #[error("Git error: {0}")]
    GixCommit(#[from] gix::object::commit::Error),
    #[error("Invalid time. (time went backwards or way way way too far forwards (hello future! whats it like?))")]
    InvalidTime,
    #[error("Invalid timezone provided. (where are you?)")]
    InvalidTimeZone,
}

/// Bucket result type
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
/// A bucket
pub struct Bucket {
    bucket_path: PathBuf,
}

impl Bucket {
    /// Open a bucket from its name
    ///
    /// # Errors
    /// - Bucket does not exist
    pub fn from_name<C>(ctx: &impl ScoopContext<C>, name: impl AsRef<Path>) -> Result<Self> {
        Self::from_path(ctx.buckets_path().join(name))
    }

    /// Open given path as a bucket
    ///
    /// # Errors
    /// - Bucket does not exist
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let bucket_path = path.as_ref().to_path_buf();

        if bucket_path.exists() {
            Ok(Self { bucket_path })
        } else {
            Err(Error::InvalidBucket(path.as_ref().to_path_buf()))
        }
    }

    /// Open a single bucket, or return all available buckets
    ///
    /// # Errors
    /// - Any listed or provided bucket is invalid
    /// - Unable to read the bucket directory
    pub fn one_or_all<C>(
        ctx: &impl ScoopContext<C>,
        name: Option<impl AsRef<Path>>,
    ) -> Result<Vec<Self>> {
        if let Some(name) = name {
            Ok(vec![Bucket::from_name(ctx, name)?])
        } else {
            Bucket::list_all(ctx)
        }
    }

    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    #[inline]
    pub fn open_repo(&self) -> Result<Repo> {
        Ok(Repo::from_bucket(self)?)
    }

    /// Gets the bucket's name (the final component of the path)
    ///
    /// # Panics
    /// If the `file_name` function returns `None`, or a non-utf8 string.
    #[must_use]
    pub fn name(&self) -> Cow<'_, str> {
        self.path()
            .file_name()
            .map(|f| f.to_string_lossy())
            .expect("File to have file name")
    }

    #[must_use]
    /// Gets the bucket's path
    pub fn path(&self) -> &Path {
        &self.bucket_path
    }

    /// Gets all buckets
    ///
    /// # Errors
    /// - Was unable to read the bucket directory
    /// - Any listed bucket is invalid
    pub fn list_all<C>(ctx: &impl ScoopContext<C>) -> Result<Vec<Bucket>> {
        let bucket_dir = std::fs::read_dir(ctx.buckets_path())?;

        bucket_dir
            .filter(|entry| entry.as_ref().is_ok_and(|entry| entry.path().is_dir()))
            .map(|entry| Self::from_path(entry?.path()))
            .collect()
    }

    /// List all packages contained within this bucket
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - Any package has an invalid path or invalid contents
    /// - See more at [`packages::Error`]
    pub fn list_packages(&self) -> packages::Result<Vec<Manifest>> {
        let packages = self.list_package_paths()?;

        // TODO: Use rayon here
        packages.into_iter().map(Manifest::from_path).collect()
    }

    /// List all packages contained within this bucket, ignoring invalid buckets
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - See more at [`packages::Error`]
    pub fn list_packages_unchecked(&self) -> packages::Result<Vec<Manifest>> {
        let packages = self.list_package_paths()?;

        // TODO: Use rayon here
        Ok(packages
            .into_iter()
            .filter_map(|path| Manifest::from_path(path).ok())
            .collect())
    }

    /// List all packages contained within this bucket, returning their names
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - See more at [`packages::Error`]
    pub fn list_package_paths(&self) -> packages::Result<Vec<PathBuf>> {
        enum BucketPath {
            Valid(PathBuf),
            Invalid(PathBuf),
        }

        impl BucketPath {
            fn is_valid(&self) -> bool {
                matches!(self, BucketPath::Valid(_))
            }
        }

        impl AsRef<Path> for BucketPath {
            fn as_ref(&self) -> &Path {
                match self {
                    BucketPath::Invalid(path) | BucketPath::Valid(path) => path,
                }
            }
        }

        let bucket_path = {
            let bucket_path = self.path().join("bucket");

            if bucket_path.exists() {
                BucketPath::Valid(bucket_path)
            } else {
                BucketPath::Invalid(self.path().to_owned())
            }
        };

        let dir = std::fs::read_dir(&bucket_path)?;

        let paths = dir.map(|entry| entry.map(|file| file.path()).map_err(packages::Error::from));

        if bucket_path.is_valid() {
            paths.collect()
        } else {
            paths
                .filter_map(|path| match path {
                    Ok(path) if path.extension() == Some(OsStr::new("json")) => Some(Ok(path)),
                    Err(e) => Some(Err(e)),
                    _ => None,
                })
                .collect()
        }
    }

    /// List all packages contained within this bucket, returning their names
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - See more at [`packages::Error`]
    pub fn list_package_names(&self) -> packages::Result<Vec<String>> {
        let packages = self.list_package_paths()?;

        Ok(packages
            .into_iter()
            .filter_map(|path| {
                path.with_extension("")
                    .file_name()
                    .map(|file_name| file_name.to_string_lossy().to_string())
            })
            .collect())
    }

    /// Get the path to the manifest for the given package name
    pub fn get_manifest_path(&self, name: impl AsRef<str>) -> PathBuf {
        let buckets_path = self.path();
        let manifests_path = buckets_path.join("bucket");

        let file_name = format!("{}.json", name.as_ref());

        manifests_path.join(file_name)
    }

    /// Gets the manifest that represents the given package name
    ///
    /// # Errors
    /// - Could not load the manifest from the path
    pub fn get_manifest(&self, name: impl AsRef<str>) -> packages::Result<Manifest> {
        let manifest_path = self.get_manifest_path(name);

        Manifest::from_path(manifest_path).map(|manifest| manifest.with_bucket(self))
    }

    /// List all matches for the given pattern
    ///
    /// # Errors
    /// - Could not load the manifest from the path
    pub fn matches<'a, C: ScoopContext<config::Scoop>>(
        &self,
        ctx: &'a C,
        installed_only: bool,
        search_regex: &Regex,
        search_mode: SearchMode,
    ) -> packages::Result<Vec<Manifest>>
    where
        &'a C: Send + Sync,
    {
        // Ignore loose files in the buckets dir
        if !self.path().is_dir() {
            return Ok(vec![]);
        }

        let bucket_contents = self.list_package_names()?;

        let matches = bucket_contents
            .par_iter()
            .filter_map(|manifest_name| {
                // Ignore non-matching manifests
                if search_mode.eager_name_matches(manifest_name, search_regex) {
                    let manifest = self.get_manifest(manifest_name).ok()?;

                    if !installed_only || manifest.is_installed(ctx, Some(&self.name())) {
                        Some(manifest)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(matches)
    }

    /// List all used buckets
    ///
    /// # Errors
    /// Invalid install manifest
    /// Reading directories fails
    pub fn used(ctx: &impl ScoopContext<config::Scoop>) -> packages::Result<HashSet<String>> {
        Ok(InstallManifest::list_all(ctx)?
            .par_iter()
            .filter_map(|entry| entry.bucket.clone())
            .collect())
    }

    // TODO: Check if calling this for every single bucket is slow
    /// Check if the current bucket is used
    ///
    /// # Errors
    /// Invalid install manifest
    /// Reading directories fails
    pub fn is_used(&self, ctx: &impl ScoopContext<config::Scoop>) -> packages::Result<bool> {
        Ok(Self::used(ctx)?.contains(&self.name().to_string()))
    }

    /// Checks if the given bucket is outdated
    ///
    /// # Errors
    /// - The bucket could not be opened as a directory
    /// - No remote named "origin"
    /// - No active branch
    /// - No reference "`FETCH_HEAD`"
    pub fn outdated(&self) -> Result<bool> {
        Ok(self.open_repo()?.outdated()?)
    }

    /// Get the number of manifests in the bucket
    ///
    /// # Errors
    /// - Could not read the bucket directory
    pub fn manifests(&self) -> packages::Result<usize> {
        Ok(self.list_package_paths()?.len())
    }

    #[deprecated(note = "Use `manifests` instead. This function is much slower")]
    #[cfg(not(feature = "v2"))]
    /// Get the number of manifests in the bucket using async I/O
    ///
    /// # Errors
    /// - Could not read the bucket directory
    pub async fn manifests_async(&self) -> Result<usize> {
        let mut read_dir = tokio::fs::read_dir(self.path().join("bucket")).await?;
        let mut count = 0;

        while (read_dir.next_entry().await?).is_some() {
            count += 1;
        }

        Ok(count)
    }

    /// Get the bucket's source url
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    /// - The bucket's origin remote could not be found
    /// - The remote's url is not utf8
    /// - The remote's url is not set
    pub fn source(&self) -> Result<String> {
        Ok(self
            .open_repo()?
            .origin()
            .ok_or(git::Error::MissingRemote("origin".to_string()))?
            .url(gix::remote::Direction::Fetch)
            .map(std::string::ToString::to_string)
            .ok_or(git::Error::NonUtf8)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::contexts::User;

    use super::*;

    #[test]
    fn test_list_all_buckets() {
        let ctx = User::new();
        let buckets = Bucket::list_all(&ctx).unwrap();

        assert!(!buckets.is_empty());
    }

    #[test]
    fn test_main_bucket_update() {
        let ctx = User::new();

        let bucket = Bucket::from_name(&ctx, "main").unwrap();

        bucket
            .open_repo()
            .unwrap()
            .pull(&User::new(), None)
            .unwrap();
    }

    #[test]
    fn test_extras_bucket_update() {
        let ctx = User::new();

        let bucket = Bucket::from_name(&ctx, "extras").unwrap();

        bucket
            .open_repo()
            .unwrap()
            .pull(&User::new(), None)
            .unwrap();
    }
}
