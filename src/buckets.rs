use std::path::PathBuf;

use crate::{
    get_scoop_path,
    packages::{FromPath, Manifest},
};

pub struct Bucket {
    name: String,
}

impl Bucket {
    pub fn new(name: String) -> Self {
        Self { name }
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

    #[must_use]
    /// Get the paths where buckets are stored
    pub fn get_buckets_path() -> PathBuf {
        let scoop_path = get_scoop_path();

        scoop_path.join("buckets")
    }

    pub fn get_path(&self) -> PathBuf {
        Self::get_buckets_path().join(&self.name)
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
