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

// Ensure supported environment
#[cfg(all(not(docsrs), not(windows), not(feature = "unstable_linux")))]
compile_error!(
    "Only Windows is supported at the moment.\nSee https://github.com/winpax/sprinkles/issues/111 for more information."
);

#[cfg(not(windows))]
#[allow(clippy::diverging_sub_expression)]
macro_rules! windows_only {
    () => {
        unimplemented!("Not implemented on non-windows platforms")
    };
}

use std::{fmt, str::FromStr};

use quork::traits::list::ListVariants;
use serde::{Deserialize, Serialize};

pub mod buckets;
#[cfg(feature = "manifest-hashes")]
pub mod cache;
pub mod config;
pub mod contexts;
pub mod git;
pub mod handles;
#[cfg(feature = "manifest-hashes")]
pub mod hash;
pub mod packages;
pub mod progress;
pub mod proxy;
pub mod requests;
pub mod scripts;
pub mod shell;
pub mod version;

mod env;
mod system;

#[macro_use]
extern crate log;

use contexts::Error;

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

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        contexts::{ScoopContext, User},
        packages::{self, CreateManifest, InstallManifest},
    };

    #[test]
    fn test_list_install_manifests() {
        let ctx = User::new().unwrap();
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
