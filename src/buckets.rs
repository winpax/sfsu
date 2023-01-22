use std::path::PathBuf;

use crate::{
    get_scoop_path,
    packages::{FromPath, Manifest},
};

pub struct Bucket {
    pub name: String,
}

impl Bucket {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_owned(),
        }
    }

    /// Open the given path as a bucket
    ///
    /// # Errors
    /// - The directory could not be opened as a git repository
    pub fn open() {
        unimplemented!()
    }

    /// Update the bucket by pulling any changes
    ///
    /// # Errors
    pub fn update(&self) {
        unimplemented!()
    }

    /// Get the remote url of the bucket
    ///
    /// # Errors
    /// - The remote "origin" could not be retrieved
    ///
    /// # Panics
    /// - The remote url was missing
    pub fn get_remote(&self) {
        unimplemented!()
    }

    /// Get the paths where buckets are stored
    pub fn get_buckets_path() -> PathBuf {
        let scoop_path = get_scoop_path();

        scoop_path.join("buckets")
    }

    pub fn get_path(&self) -> PathBuf {
        Self::get_buckets_path().join(&self.name)
    }

    pub fn get_all() -> std::io::Result<Vec<Bucket>> {
        let buckets_path = Self::get_buckets_path();

        let bucket_dir = std::fs::read_dir(buckets_path)?;

        let buckets = bucket_dir.map(|entry| {
            let entry = entry?;
            let name = {
                let name = entry.file_name();

                name.to_string_lossy().to_string()
            };

            Ok::<Bucket, std::io::Error>(Self { name })
        });

        let buckets = buckets.collect::<Result<Vec<_>, _>>()?;

        Ok(buckets)
    }

    /// Gets the manifest that represents the given package name
    pub fn get_manifest(&self, name: impl AsRef<str>) -> std::io::Result<Manifest> {
        let buckets_path = self.get_path();
        let manifests_path = buckets_path.join("bucket");

        let file_name = format!("{}.json", name.as_ref());

        let manifest_path = manifests_path.join(file_name);

        Manifest::from_path(manifest_path)
    }
}
