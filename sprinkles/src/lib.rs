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

#[macro_use]
extern crate log;

use contexts::Error;

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
