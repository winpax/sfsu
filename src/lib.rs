#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

use std::{ffi::OsStr, path::PathBuf};

use rayon::prelude::*;

pub mod buckets;
pub mod config;
pub mod packages;

mod opt;
/// Currently this is mostly an internal api
pub mod output;

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

pub struct Scoop;

impl Scoop {
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
}
