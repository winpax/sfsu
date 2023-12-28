//! Summary info structs

use std::{num::TryFromIntError, time::SystemTimeError};

use serde::{Deserialize, Serialize};

use crate::{
    buckets::{BucketError, RepoError},
    packages::PackageError,
};

pub mod bucket;
pub mod package;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summaries {
    pub buckets: Vec<bucket::Summary>,
    pub packages: Vec<package::Summary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<crate::config::Scoop>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    #[error("System IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Time went backwards by {0}")]
    TimeWentBackwards(#[from] SystemTimeError),
    #[error("Parsing int into int failed")]
    TryFromInt(#[from] TryFromIntError),
    #[error("Main remote is missing a url")]
    MissingRemoteUrl,
    #[error("The produced Date Time was invalid. Try again later, or report this issue.")]
    InvalidDateTime,
    #[error("Package had a missing or invalid file name (i.e path terminated with '..'")]
    MissingOrInvalidFileName,
}

pub type Result<T> = std::result::Result<T, Error>;
