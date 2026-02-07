use std::path::{Path, PathBuf};

use crate::{config, git, system::paths::Paths};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Global Context Errors
pub enum Error {
    #[error("Failed to find real path to scoop -> IO Error: {0}")]
    CanonPath(std::io::Error),
    #[error("Failed to load Scoop config -> IO Error: {0}")]
    LoadingConfig(std::io::Error),
    #[error("Scoop path does not exist. Looked at {0}")]
    MissingScoopPath(PathBuf),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
/// User's Scoop install adapter
pub struct User {
    config: config::Scoop,
    path: PathBuf,
}

impl User {
    /// Construct a new user context adapter
    pub fn new() -> Result<Self> {
        let config = config::Scoop::load().map_err(Error::LoadingConfig)?;

        let path = {
            if let Some(path) = crate::env::paths::scoop_path() {
                path
            } else {
                // If not provided in the config, this will default to the <user's home directory>/scoop
                config.root_path
            }
        };

        let path = if path.exists() {
            dunce::canonicalize(path).map_err(Error::CanonPath)?
        } else {
            return Err(Error::MissingScoopPath(path));
        };

        let config = config::Scoop::load().map_err(Error::LoadingConfig)?;

        Ok(Self { config, path })
    }
}

impl super::ScoopContext for User {
    type Config = config::Scoop;

    const APP_NAME: &'static str = "scoop";
    const CONTEXT_NAME: &'static str = "user";
    const ELEVATED: bool = false;

    /// Load the Scoop configuration
    ///
    /// # Errors
    /// - Could not load the configuration
    fn config(&self) -> &config::Scoop {
        &self.config
    }

    fn config_mut(&mut self) -> &mut config::Scoop {
        &mut self.config
    }

    fn symlinks_enabled(&self) -> bool {
        !self.config.no_junction
    }

    fn proxy(&self) -> Option<&crate::proxy::Proxy> {
        self.config().proxy.as_ref()
    }

    /// Gets the user's scoop path, via either the default path or as provided by the SCOOP env variable
    ///
    /// Will ignore the global scoop path
    ///
    /// # Panics
    /// - There is no home folder
    /// - The discovered scoop path does not exist
    fn path(&self) -> &Path {
        &self.path
    }

    /// Gets the user's scoop cache path
    fn cache_path(&self) -> PathBuf {
        if let Some(cache_path) = crate::env::paths::scoop_cache() {
            cache_path
        } else if let Some(cache_path) = self.config().cache_path.as_ref() {
            cache_path.clone()
        } else {
            self.sub_path("cache")
        }
    }

    /// Get the path to the log directory
    ///
    /// By default, this will be the user's "%LocalAppData%/sfsu/logs" directory,
    /// or, in the case of a debug build, "<current working directory>/logs".
    ///
    /// # Errors
    /// - Creating the directory fails
    fn logging_dir(&self) -> std::io::Result<PathBuf> {
        #[cfg(not(debug_assertions))]
        let logs_path = self.apps_path().join("sfsu").join("current").join("logs");

        #[cfg(debug_assertions)]
        let logs_path: PathBuf = Paths::LocalAppData
            .into_path()
            .or_else(|| std::env::var("LocalAppData").ok().map(Into::into))
            .expect("either windows defined local app data or env var `LocalAppData`")
            .join("sfsu")
            .join("logs");

        if !logs_path.exists() {
            std::fs::create_dir_all(&logs_path)?;
        }

        Ok(logs_path)
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    fn open_repo(&self) -> Option<git::Result<git::Repo>> {
        Some(git::Repo::scoop_app(self))
    }

    /// Check if Scoop is outdated
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    /// - The Scoop app could not be checked for updates
    async fn outdated(&self) -> super::Result<bool> {
        let config = self.config();
        let scoop_repo = self.open_repo().expect("scoop repo")?;

        let current_branch = scoop_repo.current_branch()?;
        let scoop_config_branch = config.scoop_branch.name();

        if current_branch != scoop_config_branch {
            scoop_repo.checkout(scoop_config_branch)?;
            debug!("Switched to branch {scoop_config_branch}");
            return Ok(true);
        }

        Ok(scoop_repo.outdated()?)
    }
}
