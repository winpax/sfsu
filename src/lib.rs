#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

use std::{ffi::OsStr, fmt::Display, path::PathBuf};

use colored::Colorize;
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

/// This is a workaround for type equality constraints <https://github.com/rust-lang/rust/issues/20041>
pub(crate) trait TyEq {}

impl<T> TyEq for (T, T) {}

impl Scoop {
    #[must_use]
    /// Gets the user's scoop path, via either the default path or as provided by the SCOOP env variable
    ///
    /// Will ignore the global scoop path
    ///
    /// # Panics
    /// - There is no home folder
    /// - The discovered scoop path does not exist
    pub fn get_scoop_path() -> PathBuf {
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

    /// List all scoop apps and return their paths
    ///
    /// # Errors
    /// - Reading dir fails
    ///
    /// # Panics
    /// - Reading dir fails
    pub fn list_installed_scoop_apps() -> std::io::Result<Vec<PathBuf>> {
        let scoop_apps_path = Self::get_scoop_path().join("apps");

        let read = scoop_apps_path.read_dir()?;

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

#[deprecated(note = "Use `sfsu::deprecate` instead")]
pub trait Deprecateable {
    fn is_deprecated() -> bool;

    #[must_use]
    fn deprecation_message() -> Option<String> {
        None
    }

    fn print_deprecation_message() {
        if Self::is_deprecated() {
            eprint!("WARNING: This command is deprecated");
            if let Some(message) = Self::deprecation_message() {
                eprint!(": {message}");
            }
            eprintln!();
        }
    }
}

pub fn deprecate(message: impl Display) {
    eprintln!(
        "{}",
        format!("WARNING: This command is deprecated: {message}").yellow()
    );
}
