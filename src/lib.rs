use std::path::PathBuf;

// TODO: Global custom hook fn

pub mod config;

pub fn get_scoop_path() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| panic!("Could not find home directory"));

    home_dir.join("scoop")
}

pub mod buckets;

pub mod packages;

pub fn get_powershell_path() -> anyhow::Result<PathBuf> {
    use which::which;

    if let Ok(path) = which("pwsh") {
        Ok(path)
    } else if let Ok(path) = which("powershell") {
        Ok(path)
    } else {
        Err(anyhow::anyhow!("Could not find powershell"))
    }
}
