use std::{
    borrow::Cow,
    collections::HashSet,
    path::{Path, PathBuf},
};

use git2::{Remote, Repository};
use rayon::prelude::*;
use regex::Regex;

use crate::{
    output::sectioned::{Children, Section, Text},
    packages::{self, CreateManifest, InstallManifest, Manifest, SearchMode},
    Scoop,
};

#[derive(Debug, thiserror::Error)]
pub enum BucketError {
    #[error("Interacting with repo: {0}")]
    RepoError(#[from] RepoError),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("The bucket \"{0}\" does not exist")]
    InvalidBucket(PathBuf),
}

pub type Result<T> = std::result::Result<T, BucketError>;

#[derive(Debug, Clone)]
pub struct Bucket {
    bucket_path: PathBuf,
}

impl Bucket {
    /// Open a bucket from its name
    ///
    /// # Errors
    /// - Bucket does not exist
    pub fn new(name: impl AsRef<Path>) -> Result<Self> {
        Self::open(Scoop::buckets_path().join(name))
    }

    /// Open given path as a bucket
    ///
    /// # Errors
    /// - Bucket does not exist
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let bucket_path = path.as_ref().to_path_buf();

        if bucket_path.exists() {
            Ok(Self { bucket_path })
        } else {
            Err(BucketError::InvalidBucket(path.as_ref().to_path_buf()))
        }
    }

    /// Open a single bucket, or return all available buckets
    ///
    /// # Errors
    /// - Any listed or provided bucket is invalid
    /// - Unable to read the bucket directory
    pub fn one_or_all(name: Option<impl AsRef<Path>>) -> Result<Vec<Self>> {
        if let Some(name) = name {
            Ok(vec![Bucket::new(name)?])
        } else {
            Bucket::list_all()
        }
    }

    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    #[inline]
    pub fn open_repo(&self) -> Result<BucketRepo> {
        Ok(BucketRepo::from_bucket(self)?)
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
    pub fn path(&self) -> &Path {
        &self.bucket_path
    }

    /// Gets all buckets
    ///
    /// # Errors
    /// - Was unable to read the bucket directory
    /// - Any listed bucket is invalid
    pub fn list_all() -> Result<Vec<Bucket>> {
        let bucket_dir = std::fs::read_dir(Scoop::buckets_path())?;

        bucket_dir
            .filter(|entry| entry.as_ref().is_ok_and(|entry| entry.path().is_dir()))
            .map(|entry| Self::new(entry?.path()))
            .collect()
    }

    /// List all packages contained within this bucket
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - Any package has an invalid path or invalid contents
    /// - See more at [`packages::PackageError`]
    pub fn list_packages(&self) -> packages::Result<Vec<Manifest>> {
        let dir = std::fs::read_dir(self.path().join("bucket"))?;

        dir.map(|manifest| Manifest::from_path(manifest?.path()))
            .collect()
    }

    /// List all packages contained within this bucket, ignoring errors
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - See more at [`packages::PackageError`]
    pub fn list_packages_unchecked(&self) -> packages::Result<Vec<Manifest>> {
        let dir = std::fs::read_dir(self.path().join("bucket"))?;

        Ok(dir
            .map(|manifest| Manifest::from_path(manifest?.path()))
            .filter_map(|result| match result {
                Ok(v) => Some(v),
                Err(_) => None,
            })
            .collect())
    }

    /// List all packages contained within this bucket, returning their names
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - See more at [`packages::PackageError`]
    pub fn list_package_names(&self) -> packages::Result<Vec<String>> {
        let dir = std::fs::read_dir(self.path().join("bucket"))?;

        Ok(dir
            .map(|entry| {
                entry.map(|file| {
                    file.path()
                        .with_extension("")
                        .file_name()
                        .map(|file_name| file_name.to_string_lossy().to_string())
                })
            })
            .filter_map(|file_name| match file_name {
                Ok(Some(file_name)) => Some(file_name),
                _ => None,
            })
            .collect())
    }

    /// Gets the manifest that represents the given package name
    ///
    /// # Errors
    /// - Could not load the manifest from the path
    pub fn get_manifest(&self, name: impl AsRef<str>) -> packages::Result<Manifest> {
        let buckets_path = self.path();
        let manifests_path = buckets_path.join("bucket");

        let file_name = format!("{}.json", name.as_ref());

        let manifest_path = manifests_path.join(file_name);

        Manifest::from_path(manifest_path).map(|manifest| manifest.with_bucket(self))
    }

    /// List all matches for the given pattern
    ///
    /// # Errors
    /// - Could not load the manifest from the path
    pub fn matches(
        &self,
        search_regex: &Regex,
        search_mode: SearchMode,
    ) -> packages::Result<Option<Section<Section<Text<String>>>>> {
        // Ignore loose files in the buckets dir
        if !self.path().is_dir() {
            return Ok(None);
        }

        let bucket_contents = self.list_package_names()?;

        let matches = bucket_contents
            .par_iter()
            .filter_map(|manifest_name| {
                // Ignore non-matching manifests
                if search_mode.eager_name_matches(manifest_name, search_regex) {
                    match self.get_manifest(manifest_name) {
                        Ok(manifest) => {
                            manifest.parse_output(self.name(), false, search_regex, search_mode)
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if matches.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                Section::new(Children::from(matches))
                    // TODO: Remove quotes and bold bucket name
                    .with_title(format!("'{}' bucket:", self.name())),
            ))
        }
    }

    /// List all used buckets
    ///
    /// # Errors
    /// Invalid install manifest
    /// Reading directories fails
    pub fn used() -> packages::Result<HashSet<String>> {
        Ok(InstallManifest::list_all()?
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
    pub fn is_used(&self) -> packages::Result<bool> {
        Ok(Self::used()?.contains(&self.name().to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("Could not find the active branch (HEAD)")]
    NoActiveBranch,

    #[error("Git error: {0}")]
    Git2(#[from] git2::Error),
}

pub type RepoResult<T> = std::result::Result<T, RepoError>;

pub struct BucketRepo {
    bucket: Bucket,
    repo: Repository,
}

impl BucketRepo {
    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn from_bucket(bucket: &Bucket) -> RepoResult<Self> {
        let bucket = bucket.clone();

        let repo = Repository::open(bucket.path())?;

        Ok(Self { bucket, repo })
    }

    /// Get the current remote branch
    ///
    /// # Errors
    /// - Missing head
    ///
    /// # Panics
    /// - Non-utf8 branch name
    pub fn main_remote(&self) -> RepoResult<Remote<'_>> {
        Ok(self
            .repo
            .find_remote(self.repo.head()?.name().expect("utf8 branch name"))?)
    }

    /// Checks if the bucket is outdated
    pub fn outdated(&self) -> RepoResult<bool> {
        // let main_remote = self.main_remote()?;
        // self.repo.diff_tree_to_workdir(main_remote, None);
        unimplemented!()
    }

    /// Update the bucket by pulling any changes
    pub fn update(&self) {
        unimplemented!()
    }

    /// Get the remote url of the bucket
    pub fn get_remote(&self) {
        unimplemented!()
    }
}
