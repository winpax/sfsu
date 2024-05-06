use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::git;

mod global;
mod user;

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
}

/// An adapter for scoop contexts
///
/// This is used to provide a common interface for scoop contexts, and to allow for mocking in tests
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
    /// Load the Scoop configuration
    ///
    /// # Errors
    /// - Could not load the configuration
    fn config() -> std::io::Result<C>;

    /// Get the git executable path
    ///
    /// # Errors
    /// - Could not find `git` in path
    fn git_path() -> Result<PathBuf, which::Error>;

    #[must_use]
    /// Gets the user's scoop path, via either the default path or as provided by the SCOOP env variable
    ///
    /// Will ignore the global scoop path
    ///
    /// # Panics
    /// - There is no home folder
    /// - The discovered scoop path does not exist
    fn path() -> PathBuf;

    fn scoop_sub_path(segment: impl AsRef<Path>) -> PathBuf;

    #[must_use]
    /// Gets the user's scoop apps path
    fn apps_path() -> PathBuf;

    #[must_use]
    /// Gets the user's scoop buckets path
    fn buckets_path() -> PathBuf;

    #[must_use]
    /// Gets the user's scoop cache path
    fn cache_path() -> PathBuf;

    #[must_use]
    /// Gets the user's scoop persist path
    fn persist_path() -> PathBuf;

    #[must_use]
    /// Gets the user's scoop shims path
    fn shims_path() -> PathBuf;

    #[must_use]
    /// Gets the user's scoop workspace path
    fn workspace_path() -> PathBuf;

    /// List all scoop apps and return their paths
    ///
    /// # Errors
    /// - Reading dir fails
    ///
    /// # Panics
    /// - Reading dir fails
    fn installed_apps() -> std::io::Result<Vec<PathBuf>>;

    /// Get the path to the log directory
    ///
    /// # Errors
    /// - Creating the directory fails
    fn logging_dir() -> std::io::Result<PathBuf>;

    /// Create a new log file
    ///
    /// # Errors
    /// - Creating the file fails
    ///
    /// # Panics
    /// - Could not convert tokio file into std file
    async fn new_log() -> Result<File, Error>;

    /// Create a new log file
    ///
    /// This function is synchronous and does not allow for timeouts.
    /// If for some reason there are no available log files, this function will block indefinitely.
    ///
    /// # Errors
    /// - Creating the file fails
    fn new_log_sync() -> Result<File, Error>;

    /// Checks if the app is installed by its name
    ///
    /// # Errors
    /// - Reading app dir fails
    fn app_installed(name: impl AsRef<str>) -> std::io::Result<bool>;

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    fn open_repo() -> git::Result<git::Repo>;

    /// Check if Scoop is outdated
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    /// - The Scoop app could not be checked for updates
    fn outdated() -> Result<bool, Error>;
}
