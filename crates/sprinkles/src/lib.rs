#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    rustdoc::all,
    rust_2024_compatibility,
    missing_docs
)]
#![allow(clippy::module_name_repetitions)]

use std::{fmt, str::FromStr};

use quork::traits::list::ListVariants;
use serde::{Deserialize, Serialize};

pub use semver;

pub mod buckets;
#[cfg(feature = "manifest-hashes")]
pub mod cache;
pub mod config;
pub mod contexts;
pub mod diagnostics;
pub mod git;
#[doc(hidden)]
pub mod hacks;
#[cfg(feature = "manifest-hashes")]
pub mod hash;
pub mod packages;
pub mod progress;
pub mod proxy;
pub mod requests;
pub mod scripts;
pub mod shell;
#[cfg(not(feature = "v2"))]
pub mod stream;
pub mod version;
pub mod wrappers;

mod env;

#[doc(hidden)]
pub mod __versions {
    //! Version information

    /// Sprinkles library version
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    #[must_use]
    /// Get the git2 library version
    pub fn git2_version() -> git2::Version {
        git2::Version::get()
    }
}

#[macro_use]
extern crate log;

use contexts::Error;

/// Ensure supported environment
mod const_assertions {
    const _: () = assert!(cfg!(windows), "Only windows is supported");
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ListVariants)]
/// Supported architectures
pub enum Architecture {
    /// 64 bit Arm
    Arm64,
    /// 64 bit
    #[serde(rename = "64bit")]
    X64,
    #[serde(rename = "32bit")]
    /// 32 bit
    X86,
}

impl FromStr for Architecture {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "64bit" => Self::X64,
            "32bit" => Self::X86,
            "arm64" => Self::Arm64,
            _ => return Err(Error::UnsupportedArchitecture),
        })
    }
}

impl Architecture {
    /// Get the architecture of the current environment
    pub const ARCH: Self = Self::X64;

    #[must_use]
    /// Get the architecture of the current environment
    ///
    /// # Panics
    /// - Unsupported environment
    pub const fn from_env() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::X64
        } else if cfg!(target_arch = "x86") {
            Self::X86
        } else if cfg!(target_arch = "aarch64") {
            Self::Arm64
        } else {
            panic!("Unsupported architecture")
        }
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

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arm64 => write!(f, "arm64"),
            Self::X64 => write!(f, "64bit"),
            Self::X86 => write!(f, "32bit"),
        }
    }
}

impl Default for Architecture {
    fn default() -> Self {
        Self::from_env()
    }
}

#[deprecated(note = "Use `contexts::User` instead")]
#[cfg(not(feature = "v2"))]
/// Alias for [`contexts::User`]
pub type Scoop = contexts::User;

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        contexts::{ScoopContext, User},
        packages::{self, CreateManifest, InstallManifest},
    };

    #[test]
    fn test_list_install_manifests() {
        let ctx = User::new();
        let app_paths = ctx.installed_apps().unwrap();

        app_paths
            .into_iter()
            .filter_map(|path| {
                let path = path.join("current/install.json");
                let result = InstallManifest::from_path(path);

                match result {
                    Ok(v) => Some(v),
                    // These are really the only errors we care about
                    Err(packages::Error::ParsingManifest(name, err)) => panic!("{name}: {err}"),
                    Err(_) => None,
                }
            })
            .collect_vec();
    }
}
