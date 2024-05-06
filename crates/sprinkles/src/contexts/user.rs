use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
};

use chrono::Local;
use rayon::prelude::*;

use crate::{abandon, config, contexts::Error, git};

/// The Scoop install reference
pub struct User;

impl super::ScoopContext<config::Scoop> for User {
    /// Load the Scoop configuration
    ///
    /// # Errors
    /// - Could not load the configuration
    fn config() -> std::io::Result<config::Scoop> {
        config::Scoop::load()
    }

    /// Get the git executable path
    ///
    /// # Errors
    /// - Could not find `git` in path
    fn git_path() -> Result<PathBuf, which::Error> {
        which::which("git")
    }

    #[must_use]
    /// Gets the user's scoop path, via either the default path or as provided by the SCOOP env variable
    ///
    /// Will ignore the global scoop path
    ///
    /// # Panics
    /// - There is no home folder
    /// - The discovered scoop path does not exist
    fn path() -> PathBuf {
        use std::env::var_os;

        // TODO: Add support for both global and non-global scoop installs

        let scoop_path = {
            if let Some(path) = var_os("SCOOP") {
                path.into()
            } else if let Some(path) = config::Scoop::load()
                .expect("scoop config loaded correctly")
                .root_path
            {
                path.into()
            } else {
                directories::BaseDirs::new()
                    .expect("user directories")
                    .home_dir()
                    .join("scoop")
            }
        };

        if scoop_path.exists() {
            dunce::canonicalize(scoop_path).expect("failed to find real path to scoop")
        } else {
            panic!("Scoop path does not exist");
        }
    }

    fn scoop_sub_path(segment: impl AsRef<Path>) -> PathBuf {
        let path = Self::path().join(segment.as_ref());

        if !path.exists() && std::fs::create_dir_all(&path).is_err() {
            abandon!("Could not create {} directory", segment.as_ref().display());
        }

        path
    }

    #[must_use]
    /// Gets the user's scoop apps path
    fn apps_path() -> PathBuf {
        Self::scoop_sub_path("apps")
    }

    #[must_use]
    /// Gets the user's scoop buckets path
    fn buckets_path() -> PathBuf {
        Self::scoop_sub_path("buckets")
    }

    #[must_use]
    /// Gets the user's scoop cache path
    fn cache_path() -> PathBuf {
        if let Some(cache_path) = std::env::var_os("SCOOP_CACHE") {
            PathBuf::from(cache_path)
        } else if let Some(cache_path) = config::Scoop::load()
            .ok()
            .and_then(|config| config.cache_path)
        {
            cache_path
        } else {
            Self::scoop_sub_path("cache")
        }
    }

    #[must_use]
    /// Gets the user's scoop persist path
    fn persist_path() -> PathBuf {
        Self::scoop_sub_path("persist")
    }

    #[must_use]
    /// Gets the user's scoop shims path
    fn shims_path() -> PathBuf {
        Self::scoop_sub_path("shims")
    }

    #[must_use]
    /// Gets the user's scoop workspace path
    fn workspace_path() -> PathBuf {
        Self::scoop_sub_path("workspace")
    }

    /// List all scoop apps and return their paths
    ///
    /// # Errors
    /// - Reading dir fails
    ///
    /// # Panics
    /// - Reading dir fails
    fn installed_apps() -> std::io::Result<Vec<PathBuf>> {
        let apps_path = Self::apps_path();

        let read = apps_path.read_dir()?;

        Ok(read
            .par_bridge()
            .filter_map(|package| {
                let path = package.expect("valid path").path();

                // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
                if path.file_name() == Some(OsStr::new("scoop")) {
                    None
                } else {
                    Some(path)
                }
            })
            .collect())
    }

    /// Get the path to the log directory
    ///
    /// # Errors
    /// - Creating the directory fails
    fn logging_dir() -> std::io::Result<PathBuf> {
        #[cfg(not(debug_assertions))]
        let logs_path = User::apps_path().join("sfsu").join("current").join("logs");

        #[cfg(debug_assertions)]
        let logs_path = std::env::current_dir()?.join("logs");

        if !logs_path.exists() {
            std::fs::create_dir_all(&logs_path)?;
        }

        Ok(logs_path)
    }

    /// Create a new log file
    ///
    /// # Errors
    /// - Creating the file fails
    ///
    /// # Panics
    /// - Could not convert tokio file into std file
    async fn new_log() -> Result<File, Error> {
        let logs_dir = Self::logging_dir()?;
        let date = Local::now();

        let log_file = async {
            use tokio::fs::File;

            let mut i = 0;
            loop {
                i += 1;

                let log_path =
                    logs_dir.join(format!("sfsu-{}-{i}.log", date.format("%Y-%m-%d-%H-%M-%S")));

                if !log_path.exists() {
                    break File::create(log_path).await;
                }
            }
        };
        let timeout = async {
            use std::time::Duration;
            use tokio::time;

            time::sleep(Duration::from_secs(5)).await;
        };

        let log_file = tokio::select! {
            res = log_file => Ok(res),
            () = timeout => Err(Error::TimeoutCreatingLog),
        }??;

        Ok(log_file
            .try_into_std()
            .expect("converted tokio file into std file"))
    }

    /// Create a new log file
    ///
    /// This function is synchronous and does not allow for timeouts.
    /// If for some reason there are no available log files, this function will block indefinitely.
    ///
    /// # Errors
    /// - Creating the file fails
    fn new_log_sync() -> Result<File, Error> {
        let logs_dir = Self::logging_dir()?;
        let date = Local::now();

        let mut i = 0;
        let file = loop {
            i += 1;

            let log_path =
                logs_dir.join(format!("sfsu-{}-{i}.log", date.format("%Y-%m-%d-%H-%M-%S")));

            if !log_path.exists() {
                break File::create(log_path)?;
            }
        };

        Ok(file)
    }

    /// Checks if the app is installed by its name
    ///
    /// # Errors
    /// - Reading app dir fails
    fn app_installed(name: impl AsRef<str>) -> std::io::Result<bool> {
        Ok(Self::installed_apps()?
            .iter()
            .any(|path| path.file_name() == Some(OsStr::new(name.as_ref()))))
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    fn open_repo() -> Option<git::Result<git::Repo>> {
        Some(git::Repo::scoop_app())
    }

    /// Check if Scoop is outdated
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    /// - The Scoop app could not be checked for updates
    fn outdated() -> Result<bool, Error> {
        let config = config::Scoop::load()?;
        let scoop_repo = git::Repo::scoop_app()?;

        let current_branch = scoop_repo.current_branch()?;
        let scoop_config_branch = config.scoop_branch.unwrap_or("master".into());

        if current_branch != scoop_config_branch {
            scoop_repo.checkout(&scoop_config_branch)?;
            debug!("Switched to branch {}", scoop_config_branch);
            return Ok(true);
        }

        Ok(scoop_repo.outdated()?)
    }
}
