use chrono::{DateTime, Local, NaiveDateTime};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    buckets::Bucket,
    summary::{Error, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Summary {
    name: String,
    source: Url,
    updated: DateTime<Local>,
    manifests: usize,
}

impl Summary {
    /// Parse all local buckets
    ///
    /// # Errors
    /// - Listing buckets fails
    /// - Parsing buckets fails
    pub fn parse_all() -> Result<Vec<Self>> {
        Bucket::list_all()?
            .par_iter()
            .map(Self::from_bucket)
            .collect()
    }

    /// Create a bucket summary from a [`Bucket`]
    ///
    /// # Errors
    /// - Bucket repo fails to open
    /// - Bucket is missing a main remote
    /// - The main remote is missing a url
    /// - The found remote url is not a valid url
    /// - Branch missing
    /// - Branch has no commits
    /// - Found commit was not a commit
    /// - Invalid Date Time
    /// - Listing package names fails
    pub fn from_bucket(bucket: &Bucket) -> Result<Summary> {
        let repo = bucket.open_repo()?;

        let remote_url = {
            let main_remote = repo.main_remote()?;

            let Some(remote_url) = main_remote.url() else {
                return Err(Error::MissingRemoteUrl);
            };

            Url::parse(remote_url)?
        };

        let commit_time = {
            let commit_obj = repo.repo.revparse_single(&repo.branch(None)?)?;
            let commit = commit_obj.peel_to_commit()?;

            let epoch = commit.time().seconds();
            let naive_time =
                NaiveDateTime::from_timestamp_opt(epoch, 0).ok_or(Error::InvalidDateTime)?;

            naive_time
                .and_local_timezone(Local)
                .earliest()
                .ok_or(Error::InvalidDateTime)?
        };

        Ok(Self {
            name: bucket.name().to_string(),
            source: remote_url,
            updated: commit_time,
            manifests: bucket.manifest_count()?,
        })
    }
}
