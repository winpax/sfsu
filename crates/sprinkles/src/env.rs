//! Opinionated environment helpers

pub mod paths {
    #![allow(dead_code)]

    //! Paths for the scoop environment

    use std::env;

    use std::path::PathBuf;

    /// Get the Scoop location from the environment
    pub fn scoop_path() -> Option<PathBuf> {
        env::var_os("SCOOP").map(PathBuf::from)
    }

    /// Get the Scoop global location from the environment
    pub fn scoop_global() -> Option<PathBuf> {
        env::var_os("SCOOP_GLOBAL").map(PathBuf::from)
    }

    /// Get the Scoop cache location from the environment
    pub fn scoop_cache() -> Option<PathBuf> {
        env::var_os("SCOOP_CACHE").map(PathBuf::from)
    }

    pub fn config_dir() -> Option<PathBuf> {
        env::var_os("XFG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                directories::BaseDirs::new().map(|base_dirs| base_dirs.home_dir().join(".config"))
            })
    }
}
