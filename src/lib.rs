#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

use std::{ffi::OsStr, path::PathBuf};

use rayon::prelude::*;

pub mod buckets;
pub mod config;
pub mod packages;

mod opt;
/// Currently this is mostly an internal api
pub mod output;

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
pub fn list_scoop_apps() -> std::io::Result<Vec<PathBuf>> {
    let scoop_apps_path = get_scoop_path().join("apps");

    let read = scoop_apps_path.read_dir()?.collect::<Result<Vec<_>, _>>()?;

    read.par_iter()
        // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
        .filter(|package| package.path().iter().last() != Some(OsStr::new("scoop")))
        .map(|dir| dunce::realpath(dir.path()))
        .collect::<Result<_, _>>()
}
