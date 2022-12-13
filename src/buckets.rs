use std::path::{Path, PathBuf};

use crate::get_scoop_path;

pub struct Bucket {
    repo: git2::Repository,
}

impl Bucket {
    /// Open the given path as a bucket
    ///
    /// # Errors
    /// - The directory could not be opened as a git repository
    pub fn open(name: impl AsRef<Path>) -> Result<Self, git2::Error> {
        let path = get_scoop_path().join("buckets").join(name);

        let repo = git2::Repository::open(&path)?;

        Ok(Self { repo })
    }

    /// Update the bucket by pulling any changes
    ///
    /// # Errors
    pub fn update(&self) -> Result<(), git2::Error> {
        unimplemented!()
    }

    /// Get the remote url of the bucket
    ///
    /// # Errors
    /// - The remote "origin" could not be retrieved
    ///
    /// # Panics
    /// - The remote url was missing
    pub fn get_remote(&self) -> Result<String, git2::Error> {
        let remote = self.repo.find_remote("origin")?;

        Ok(remote.url().unwrap().to_string())
    }

    /// Get a list of buckets
    ///
    /// # Errors
    /// - The bucket dir could not be read
    /// - The bucket could not be read
    /// - The bucket could not be opened
    pub fn list() -> anyhow::Result<Vec<Self>> {
        let mut buckets = vec![];

        let bucket_path = get_scoop_path().join("buckets");

        for bucket in bucket_path.read_dir()? {
            let bucket = bucket?;
            let bucket_name = bucket.file_name();

            let bucket = Self::open(bucket_name)?;

            buckets.push(bucket);
        }

        Ok(buckets)
    }

    #[must_use]
    pub fn get_path() -> PathBuf {
        let scoop_path = get_scoop_path();

        scoop_path.join("buckets")
    }
}
