use std::path::PathBuf;

use crate::{
    get_scoop_path,
    packages::{CreateManifest, Manifest},
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
    pub fn open() {
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

    /// Get the paths where buckets are stored
    #[must_use]
    pub fn buckets_path() -> PathBuf {
        let scoop_path = get_scoop_path();

        scoop_path.join("buckets")
    }

    #[must_use]
    pub fn path(&self) -> PathBuf {
        Self::buckets_path().join(&self.name)
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
            .map(|entry| {
                let name = {
                    let name = entry?.file_name();

                    name.to_string_lossy().to_string()
                };

                Ok::<Bucket, std::io::Error>(Self { name })
            });

        let buckets = buckets.collect::<Result<Vec<_>, _>>()?;

        Ok(buckets)
    }

    /// Gets the manifest that represents the given package name
    ///
    /// # Errors
    /// - Could not load the manifest from the path
    pub fn get_manifest(&self, name: impl AsRef<str>) -> std::io::Result<Manifest> {
        let buckets_path = self.path();
        let manifests_path = buckets_path.join("bucket");

        let file_name = format!("{}.json", name.as_ref());

        let manifest_path = manifests_path.join(file_name);

        Manifest::from_path(manifest_path)
    }
}
