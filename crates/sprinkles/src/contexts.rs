#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

//! Scoop context adapters

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{config, git};

mod global;
mod user;

use futures::Future;
pub use global::Global;
pub use user::User;
use which::which;

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
    #[error("Error reading known buckets: {0}")]
    SerdeJson(#[from] serde_json::Error),

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
/// # use sprinkles::contexts::{ScoopContext, User};
/// let context = User::new();
/// let scoop_path = context.path();
/// ```
pub trait ScoopContext<C>: Clone + Send + Sync + 'static {
    /// The name of the context
    ///
    /// This is used internally to ignore this app when searching for installed apps,
    /// and to display the context name in the output.
    ///
    /// This string should match what the app's name is in Scoop, if applicable.
    const CONTEXT_NAME: &'static str;

    /// Get a reference to the context's configuration
    fn config(&self) -> &C;

    /// Get a mutable reference to the context's configuration
    fn config_mut(&mut self) -> &mut C;

    #[must_use]
    /// Gets the context's path
    fn path(&self) -> &Path;

    #[deprecated = "Use which::which directly instead"]
    /// Get the git executable path
    fn git_path() -> Result<PathBuf, which::Error>;

    #[must_use]
    /// Get a sub path within the context's path
    ///
    /// This function will attempt to create the path if it does not exist
    /// but will **not** panic or return an error if it fails
    fn sub_path(&self, segment: impl AsRef<Path>) -> PathBuf {
        let path = self.path().join(segment.as_ref());

        if !path.exists() {
            _ = std::fs::create_dir_all(&path);
        }

        if let Ok(dunced) = dunce::canonicalize(&path) {
            dunced
        } else {
            path
        }
    }

    #[must_use]
    /// Get the contexts's apps path
    fn apps_path(&self) -> PathBuf;

    #[must_use]
    /// Get the contexts's buckets path
    fn buckets_path(&self) -> PathBuf;

    #[must_use]
    /// Get the contexts's cache path
    fn cache_path(&self) -> PathBuf;

    #[must_use]
    /// Get the contexts's persist path
    fn persist_path(&self) -> PathBuf;

    #[must_use]
    /// Get the contexts's shims path
    fn shims_path(&self) -> PathBuf;

    #[must_use]
    /// Get the contexts's workspace path
    fn workspace_path(&self) -> PathBuf;

    #[must_use]
    /// Get the contexts's scripts path
    fn scripts_path(&self) -> PathBuf {
        self.sub_path("workspace/scripts")
    }

    /// Get the path to the log directory
    fn logging_dir(&self) -> std::io::Result<PathBuf>;

    /// List all scoop apps and return their paths
    fn installed_apps(&self) -> std::io::Result<Vec<PathBuf>> {
        use rayon::prelude::*;

        let apps_path = self.apps_path();

        let read = apps_path.read_dir()?;

        Ok(read
            .par_bridge()
            .filter_map(|package| {
                let path = package.expect("valid path").path();

                // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
                if path.file_name() == Some(OsStr::new(Self::CONTEXT_NAME)) {
                    None
                } else {
                    Some(path)
                }
            })
            .collect())
    }

    /// Checks if the app is installed by its name
    fn app_installed(&self, name: impl AsRef<str>) -> std::io::Result<bool> {
        Ok(self
            .installed_apps()?
            .iter()
            .any(|path| path.file_name() == Some(OsStr::new(name.as_ref()))))
    }

    /// Open the context's app repository, if any
    fn open_repo(&self) -> Option<git::Result<git::Repo>>;

    /// Get the path to the context's app
    ///
    /// This should return the path to the app's directory, not the repository.
    ///
    /// For example, if the context is the user context, this should return the path to the scoop app
    fn context_app_path(&self) -> PathBuf;

    /// List all known buckets
    ///
    /// This function will return a map of bucket name to url
    ///
    /// # Errors
    /// - Reading the buckets file
    /// - Deserializing the buckets file
    fn known_buckets(&self) -> &phf::Map<&'static str, &'static str> {
        &crate::buckets::known::BUCKETS
    }

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

#[derive(Debug, Clone)]
/// A little helper enum for when you want statically typed contexts, but don't know which one you want at compile time
pub enum AnyContext {
    /// The user context
    User(User),
    /// The global context
    Global(Global),
}

impl ScoopContext<config::Scoop> for AnyContext {
    const CONTEXT_NAME: &'static str = User::CONTEXT_NAME;

    fn config(&self) -> &config::Scoop {
        match self {
            AnyContext::User(user) => user.config(),
            AnyContext::Global(global) => global.config(),
        }
    }

    fn config_mut(&mut self) -> &mut config::Scoop {
        match self {
            AnyContext::User(user) => user.config_mut(),
            AnyContext::Global(global) => global.config_mut(),
        }
    }

    fn git_path() -> Result<PathBuf, which::Error> {
        which("git")
    }

    #[must_use]
    fn path(&self) -> &Path {
        match self {
            AnyContext::User(user) => user.path(),
            AnyContext::Global(global) => global.path(),
        }
    }

    fn sub_path(&self, segment: impl AsRef<Path>) -> PathBuf {
        match self {
            AnyContext::User(user) => user.sub_path(segment),
            AnyContext::Global(global) => global.sub_path(segment),
        }
    }

    fn apps_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.apps_path(),
            AnyContext::Global(global) => global.apps_path(),
        }
    }

    fn buckets_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.buckets_path(),
            AnyContext::Global(global) => global.buckets_path(),
        }
    }

    fn cache_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.cache_path(),
            AnyContext::Global(global) => global.cache_path(),
        }
    }

    fn persist_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.persist_path(),
            AnyContext::Global(global) => global.persist_path(),
        }
    }

    fn shims_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.shims_path(),
            AnyContext::Global(global) => global.shims_path(),
        }
    }

    fn workspace_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.workspace_path(),
            AnyContext::Global(global) => global.workspace_path(),
        }
    }

    fn installed_apps(&self) -> std::io::Result<Vec<PathBuf>> {
        match self {
            AnyContext::User(user) => user.installed_apps(),
            AnyContext::Global(global) => global.installed_apps(),
        }
    }

    fn logging_dir(&self) -> std::io::Result<PathBuf> {
        match self {
            AnyContext::User(user) => user.logging_dir(),
            AnyContext::Global(global) => global.logging_dir(),
        }
    }

    fn app_installed(&self, name: impl AsRef<str>) -> std::io::Result<bool> {
        match self {
            AnyContext::User(user) => user.app_installed(name),
            AnyContext::Global(global) => global.app_installed(name),
        }
    }

    fn open_repo(&self) -> Option<git::Result<git::Repo>> {
        match self {
            AnyContext::User(user) => user.open_repo(),
            AnyContext::Global(global) => global.open_repo(),
        }
    }

    fn context_app_path(&self) -> PathBuf {
        match self {
            AnyContext::User(user) => user.context_app_path(),
            AnyContext::Global(global) => global.context_app_path(),
        }
    }

    async fn outdated(&self) -> Result<bool, Error> {
        match self {
            AnyContext::User(user) => user.outdated().await,
            AnyContext::Global(global) => global.outdated().await,
        }
    }
}
