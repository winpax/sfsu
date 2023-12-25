use anyhow::Context;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    buckets::{Bucket, BucketError, RepoError},
    packages::PackageError,
};

#[derive(Debug, thiserror::Error)]
pub enum SummaryError {
    #[error("Git related error: {0}")]
    GitError(#[from] git2::Error),
    #[error("Interfacing with buckets: {0}")]
    BucketError(#[from] BucketError),
    #[error("Interfacing with repo buckets: {0}")]
    RepoError(#[from] RepoError),
    #[error("Interfacing with packages: {0}")]
    PackageError(#[from] PackageError),
    #[error("The provided remote url was invalid: {0}")]
    InvalidRemoteUrl(#[from] url::ParseError),
    #[error("Main remote is missing a url")]
    MissingRemoteUrl,
}

pub type Result<T> = std::result::Result<T, SummaryError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Summary {
    name: String,
    source: Url,
    updated: DateTime<Local>,
    manifests: usize,
}

impl Summary {
    // TODO: Replace this anyhow::Result with a better error type
    pub fn from_bucket(bucket: Bucket) -> Result<Summary> {
        let repo = bucket.open_repo()?;

        let remote_url = {
            let main_remote = repo.main_remote()?;

            let Some(remote_url) = main_remote.url() else {
                return Err(SummaryError::MissingRemoteUrl);
            };

            Url::parse(remote_url)?
        };

        let commit_time = {
            let commit_obj = repo.repo.revparse_single(&repo.branch()?)?;
            let commit = commit_obj.peel_to_commit()?;

            let epoch = commit.time().seconds();
            let naive_time = NaiveDateTime::from_timestamp_opt(epoch, 0).expect("valid datetime");

            naive_time.and_local_timezone(Local).unwrap()
        };

        Ok(Self {
            name: bucket.name().to_string(),
            source: remote_url,
            updated: commit_time,
            manifests: bucket.manifest_count()?,
        })
    }
}
