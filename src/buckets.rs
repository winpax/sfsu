use std::path::PathBuf;

use crate::get_scoop_path;

pub struct Bucket;

impl Bucket {
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
    pub fn get_path() -> PathBuf {
        let scoop_path = get_scoop_path();

        scoop_path.join("buckets")
    }
}
