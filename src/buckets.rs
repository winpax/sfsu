use std::path::{Path, PathBuf};

use git2::Repository;

use crate::{
    get_scoop_path,
    packages::{self, CreateManifest, Manifest},
};

#[derive(Debug, Clone)]
pub struct Bucket {
    bucket_path: PathBuf,
}

impl Bucket {
    #[must_use]
    pub fn new(name: impl AsRef<Path>) -> Self {
        Self::open(Self::buckets_path().join(name))
    }

    /// Open the given path as a bucket
    pub fn open(path: impl AsRef<Path>) -> Self {
        // TODO: Verify that the bucket exists and is valid
        Self {
            bucket_path: path.as_ref().to_path_buf(),
        }
    }

    /// Open the repository from the bucket path
    ///
    /// # Errors
    /// - The bucket could not be opened as a repository
    pub fn open_repo(&self) -> Result<BucketRepo, git2::Error> {
        let repo = Repository::open(self.path())?;

        Ok(BucketRepo {
            bucket: self.clone(),
            repo,
        })
    }

    /// Update the bucket by pulling any changes
    pub fn update(&self) {
        unimplemented!()
    }

    /// Get the remote url of the bucket
    pub fn get_remote(&self) {
        unimplemented!()
    }

    /// Gets the bucket's name (the final component of the path)
    ///
    /// # Panics
    /// If the `file_name` function returns `None`, or a non-utf8 string.
    #[must_use]
    pub fn name(&self) -> &str {
        self.path()
            .file_name()
            .and_then(|name| name.to_str())
            .expect("bucket to have a valid utf8 name")
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.bucket_path
    }

    /// Get the paths where buckets are stored
    #[must_use]
    pub fn buckets_path() -> PathBuf {
        let scoop_path = get_scoop_path();

        scoop_path.join("buckets")
    }

    /// Gets all buckets
    ///
    /// # Errors
    /// - Was unable to read the bucket directory
    pub fn list_all() -> std::io::Result<Vec<Bucket>> {
        let buckets_path = Self::buckets_path();

        let bucket_dir = std::fs::read_dir(buckets_path)?;

        let buckets = bucket_dir
            .filter(|entry| entry.as_ref().is_ok_and(|entry| entry.path().is_dir()))
            .map(|entry| Ok::<Bucket, std::io::Error>(Self::new(entry?.path())));

        let buckets = buckets.collect::<Result<Vec<_>, _>>()?;

        Ok(buckets)
    }

    /// List all packages contained within this bucket
    ///
    /// # Errors
    /// - The bucket is invalid
    /// - The package has an invalid path or invalid contents
    /// - See more at [`packages::PackageError`]
    pub fn list_packages(&self) -> packages::Result<Vec<Manifest>> {
        let dir = std::fs::read_dir(self.path().join("bucket"))?;

        dir.map(|manifest| Manifest::from_path(manifest?.path()))
            .collect()
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

        Manifest::from_path(manifest_path)
    }
}

pub struct BucketRepo {
    bucket: Bucket,
    repo: Repository,
}

impl BucketRepo {}
