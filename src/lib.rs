#![warn(clippy::pedantic, rust_2018_idioms)]

use std::path::PathBuf;

// TODO: Replace regex with glob
// TODO: Global custom hook fn

pub mod config;

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

    let scoop_path =
        var_os("SCOOP").map_or_else(|| dirs::home_dir().unwrap().join("scoop"), PathBuf::from);

    if scoop_path.exists() {
        dunce::canonicalize(scoop_path).expect("failed to find real path to scoop")
    } else {
        panic!("Scoop path does not exist");
    }
}

pub mod buckets;

pub mod packages;

/// Gets the powershell executable path
///
/// # Errors
/// - There is no installed powershell executable
pub fn get_powershell_path() -> anyhow::Result<PathBuf> {
    use which::which;

    if let Ok(path) = which("powershell") {
        Ok(path)
    } else {
        Err(anyhow::anyhow!("Could not find powershell"))
    }
}
