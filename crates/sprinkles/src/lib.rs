#![doc = include_str!("../README.md")]
#![warn(clippy::all, clippy::pedantic, rust_2018_idioms, missing_docs)]
#![allow(clippy::module_name_repetitions)]

use std::{ffi::OsStr, fmt, fs::File, path::PathBuf};

use chrono::Local;
use rayon::prelude::*;

pub use semver;

pub mod buckets;
pub mod calm_panic;
pub mod config;
pub mod diagnostics;
pub mod git;
pub mod output;
pub mod packages;
pub mod progress;
pub mod shell;

mod opt;

#[macro_use]
extern crate log;

/// Ensure supported environment
mod const_assertions {
    use super::Scoop;

    #[allow(unused)]
    const fn eval<T>(_: &T) {}

    const _: () = eval(&Scoop::arch());
}

/// Check if the process is elevated
///
/// # Errors
/// - Internal Windows API error
pub fn is_elevated() -> Result<bool, quork::root::Error> {
    use quork::root::is_root;

    is_root()
}

pub struct SimIter<A, B>(A, B);

impl<A: Iterator<Item = AI>, AI, B: Iterator<Item = BI>, BI> Iterator for SimIter<A, B> {
    type Item = (AI, BI);

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.0.next()?, self.1.next()?))
    }
}

pub trait KeyValue {
    fn into_pairs(self) -> (Vec<&'static str>, Vec<String>);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SupportedArch {
    Arm64,
    X64,
    X86,
}

impl SupportedArch {
    /// Get the architecture of the current environment
    pub const ARCH: Self = {
        if cfg!(target_arch = "x86_64") {
            Self::X64
        } else if cfg!(target_arch = "x86") {
            Self::X86
        } else if cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            panic!("Unsupported architecture")
        }
    };

    #[must_use]
    /// Get the architecture of the current environment
    ///
    /// # Panics
    /// - Unsupported environment
    pub const fn from_env() -> Self {
        Self::ARCH
    }

    #[must_use]
    /// Get the architecture of a given Scoop string
    ///
    /// # Panics
    /// - Unsupported environment
    pub fn from_scoop_string(string: &str) -> Self {
        match string {
            "64bit" => Self::X64,
            "32bit" => Self::X86,
            "arm64" => Self::Arm64,
            _ => panic!("Unsupported architecture"),
        }
    }
}

impl fmt::Display for SupportedArch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arm64 => write!(f, "arm64"),
            Self::X64 => write!(f, "64bit"),
            Self::X86 => write!(f, "32bit"),
        }
    }
}

pub struct Scoop;

impl Scoop {
    #[must_use]
    /// Get the system architecture
    pub const fn arch() -> SupportedArch {
        SupportedArch::from_env()
    }

    /// Get the git executable path
    ///
    /// # Errors
    /// - Could not find `git` in path
    pub fn git_path() -> Result<PathBuf, which::Error> {
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
    pub fn path() -> PathBuf {
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

    #[must_use]
    /// Gets the user's scoop apps path
    pub fn apps_path() -> PathBuf {
        Self::path().join("apps")
    }

    #[must_use]
    /// Gets the user's scoop buckets path
    pub fn buckets_path() -> PathBuf {
        Self::path().join("buckets")
    }

    /// List all scoop apps and return their paths
    ///
    /// # Errors
    /// - Reading dir fails
    ///
    /// # Panics
    /// - Reading dir fails
    pub fn installed_apps() -> std::io::Result<Vec<PathBuf>> {
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
    pub fn logging_dir() -> std::io::Result<PathBuf> {
        #[cfg(not(debug_assertions))]
        let logs_path = Scoop::apps_path().join("sfsu").join("current").join("logs");

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
    pub fn new_log() -> std::io::Result<File> {
        let logs_dir = Self::logging_dir()?;
        let date = Local::now();

        let mut i = 0;
        loop {
            i += 1;

            let log_path =
                logs_dir.join(format!("sfsu-{}-{i}.log", date.format("%Y-%m-%d-%H-%M-%S")));

            if !log_path.exists() {
                break File::create(&log_path);
            }
        }
    }

    /// Checks if the app is installed by its name
    ///
    /// # Errors
    /// - Reading app dir fails
    pub fn app_installed(name: impl AsRef<str>) -> std::io::Result<bool> {
        Ok(Self::installed_apps()?
            .iter()
            .any(|path| path.file_name() == Some(OsStr::new(name.as_ref()))))
    }

    /// Open Scoop app repository
    ///
    /// # Errors
    /// - The Scoop app could not be opened as a repository
    pub fn open_repo() -> git::Result<git::Repo> {
        git::Repo::scoop_app()
    }
}
