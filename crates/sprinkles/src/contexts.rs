#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! Scoop context adapters

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::git;

mod global;
mod user;

use futures::Future;
pub use global::Global;
pub use user::User;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Library errors
pub enum Error {
    #[error("Timeout creating new log file. This is a bug, please report it.")]
    TimeoutCreatingLog,
    #[error("Error creating log file: {0}")]
    CreatingLog(#[from] std::io::Error),
    #[error("Unsupported architecture")]
    UnsupportedArchitecture,
    #[error("Opening and interacting with Scoop repo: {0}")]
    Git(#[from] git::Error),
    #[error("Error joining task: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("{0}")]
    Custom(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    /// Map a custom error into a [`Error`]
    pub fn custom(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Custom(Box::new(err))
    }
}

/// An adapter for Scoop-like contexts
///
/// This is used to provide a common interface for Scoop-like contexts, and to allow for mocking in tests
///
/// Generally, you should not call this type directly, but instead use the [`User`] or [`Global`] types,
/// or another type that implements [`ScoopContext`].
///
/// # Example
/// ```
/// use sprinkles::contexts::{ScoopContext, User};
///
/// let scoop_path = User::path();
/// ```
pub trait ScoopContext<C> {
    /// Load the context's configuration
    fn config(&self) -> &C;

    #[must_use]
    /// Gets the context's path
    fn path(&self) -> &Path;

    /// Get the git executable path
    fn git_path() -> Result<PathBuf, which::Error>;

    /// Get a sub path within the context's path
    fn scoop_sub_path(&self, segment: impl AsRef<Path>) -> PathBuf;

    #[must_use]
    /// Gets the contexts's apps path
    fn apps_path(&self) -> PathBuf;

    #[must_use]
    /// Gets the contexts's buckets path
    fn buckets_path(&self) -> PathBuf;

    #[must_use]
    /// Gets the contexts's cache path
    fn cache_path(&self) -> PathBuf;

    #[must_use]
    /// Gets the contexts's persist path
    fn persist_path(&self) -> PathBuf;

    #[must_use]
    /// Gets the contexts's shims path
    fn shims_path(&self) -> PathBuf;

    #[must_use]
    /// Gets the contexts's workspace path
    fn workspace_path(&self) -> PathBuf;

    /// List all scoop apps and return their paths
    fn installed_apps(&self) -> std::io::Result<Vec<PathBuf>>;

    /// Get the path to the log directory
    fn logging_dir(&self) -> std::io::Result<PathBuf>;

    #[deprecated(
        note = "You should implement this yourself, as this function is inherently opinionated"
    )]
    #[cfg(not(feature = "v2"))]
    #[allow(async_fn_in_trait)]
    /// Create a new log file
    async fn new_log(&self) -> Result<File, Error>;

    #[deprecated(
        note = "You should implement this yourself, as this function is inherently opinionated"
    )]
    #[cfg(not(feature = "v2"))]
    /// Create a new log file
    ///
    /// This function is synchronous and does not allow for timeouts.
    /// If for some reason there are no available log files, this function will block indefinitely.
    fn new_log_sync(&self) -> Result<File, Error>;

    /// Checks if the app is installed by its name
    fn app_installed(&self, name: impl AsRef<str>) -> std::io::Result<bool>;

    /// Open the context's app repository, if any
    fn open_repo(&self) -> Option<git::Result<git::Repo>>;

    /// Get the path to the context's app
    ///
    /// This should return the path to the app's directory, not the repository.
    ///
    /// For example, if the context is the user context, this should return the path to the scoop app
    fn context_app_path(&self) -> PathBuf;

    #[must_use]
    /// Check if the context is outdated
    ///
    /// This may manifest in different ways, depending on the context
    ///
    /// For [`User`] and [`Global`], this will check if the Scoop repository is outdated
    /// and return `true` if it is.
    ///
    /// For implementors, this should return `true` if your provider is outdated. For example,
    /// the binary which hooks into this trait will return `true` if there is a newer version available.
    fn outdated(&self) -> impl Future<Output = Result<bool, Error>>;
}
