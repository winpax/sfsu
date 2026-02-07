use std::path::{Path, PathBuf};

use crate::{config, git};

use super::{ScoopContext, User};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Global Context Errors
pub enum Error {
    #[error("Failed to find real path to scoop -> IO Error: {0}")]
    CanonPath(std::io::Error),
    #[error("Scoop path does not exist. Looked at {0}")]
    MissingScoopPath(PathBuf),
    #[error("Failed to load User context -> {0}")]
    UserContext(#[from] super::user::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
/// Global context adapter
pub struct Global {
    path: PathBuf,
    user_context: User,
}

impl Global {
    /// Construct a new global context adapter
    ///
    /// # Errors
    /// - If the scoop global path does not exist and cannot be created
    pub fn new() -> Result<Self> {
        use std::env::var_os;

        let user_context = User::new()?;

        let path = {
            if let Some(path) = var_os("SCOOP_GLOBAL") {
                path.into()
            } else {
                user_context.config().global_path.clone()
            }
        };

        let path = if path.exists() {
            dunce::canonicalize(path).map_err(Error::CanonPath)?
        } else {
            return Err(Error::MissingScoopPath(path));
        };

        Ok(Self { path, user_context })
    }
}

impl ScoopContext for Global {
    type Config = config::Scoop;

    const APP_NAME: &'static str = User::APP_NAME;
    const CONTEXT_NAME: &'static str = "global";
    const ELEVATED: bool = true;

    fn config(&self) -> &config::Scoop {
        self.user_context.config()
    }

    fn config_mut(&mut self) -> &mut config::Scoop {
        self.user_context.config_mut()
    }

    fn symlinks_enabled(&self) -> bool {
        !self.config().no_junction
    }

    fn proxy(&self) -> Option<&crate::proxy::Proxy> {
        self.config().proxy.as_ref()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn apps_path(&self) -> PathBuf {
        self.sub_path("apps")
    }

    fn buckets_path(&self) -> PathBuf {
        self.user_context.buckets_path()
    }

    fn cache_path(&self) -> PathBuf {
        self.user_context.cache_path()
    }

    fn persist_path(&self) -> PathBuf {
        self.user_context.persist_path()
    }

    fn shims_path(&self) -> PathBuf {
        self.sub_path("shims")
    }

    fn workspace_path(&self) -> PathBuf {
        self.user_context.workspace_path()
    }

    fn logging_dir(&self) -> std::io::Result<PathBuf> {
        self.user_context.logging_dir()
    }

    fn open_repo(&self) -> Option<git::Result<git::Repo>> {
        self.user_context.open_repo()
    }

    fn context_app_path(&self) -> PathBuf {
        self.user_context.context_app_path()
    }

    async fn outdated(&self) -> Result<bool, super::Error> {
        self.user_context.outdated().await
    }
}
