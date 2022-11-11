use std::path::PathBuf;

pub mod config;

pub fn get_scoop_path() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| panic!("Could not find home directory"));

    home_dir.join("scoop")
}

pub mod buckets;

pub mod packages;
