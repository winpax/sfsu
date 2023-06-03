#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

use std::path::PathBuf;

pub mod buckets;
pub mod config;
pub mod packages;

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
            dirs::home_dir().expect("user home directory").join("scoop")
        }
    };

    if scoop_path.exists() {
        dunce::canonicalize(scoop_path).expect("failed to find real path to scoop")
    } else {
        panic!("Scoop path does not exist");
    }
}
