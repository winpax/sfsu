use std::path::{Path, PathBuf};

use crate::get_scoop_path;

pub struct Bucket {
    repo: git2::Repository,
}

impl Bucket {
    pub fn open(name: impl AsRef<Path>) -> Result<Self, git2::Error> {
        let path = get_scoop_path().join("buckets").join(name);

        let repo = git2::Repository::open(&path)?;

        Ok(Self { repo })
    }

    pub fn update(&self) -> Result<(), git2::Error> {
        unimplemented!()
    }

    pub fn get_remote(&self) -> Result<String, git2::Error> {
        let remote = self.repo.find_remote("origin")?;

        Ok(remote.url().unwrap().to_string())
    }
}

pub fn get_buckets() -> std::io::Result<Vec<Bucket>> {
    let mut buckets = vec![];

    let bucket_path = get_scoop_path().join("buckets");

    for bucket in bucket_path.read_dir()? {
        let bucket = bucket?;
        let bucket_name = bucket.file_name();

        let bucket = Bucket::open(bucket_name).unwrap();

        buckets.push(bucket);
    }

    Ok(buckets)
}

pub fn get_path() -> PathBuf {
    let scoop_path = get_scoop_path();

    scoop_path.join("buckets")
}
