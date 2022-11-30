use std::path::PathBuf;

// TODO: Replace regex with glob
// TODO: Global custom hook fn

pub mod config;

pub fn get_scoop_path() -> PathBuf {
    use std::env::var_os;

    // TODO: Add support for both global and non-global scoop installs

    let scoop_path = var_os("SCOOP")
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs::home_dir().unwrap().join("scoop"));

    if scoop_path.exists() {
        dunce::canonicalize(scoop_path).expect("failed to find real path to scoop")
    } else {
        panic!("Scoop path does not exist");
    }
}

pub mod buckets;

pub mod packages;

pub fn get_powershell_path() -> anyhow::Result<PathBuf> {
    use which::which;

    if let Ok(path) = which("powershell") {
        Ok(path)
    } else {
        Err(anyhow::anyhow!("Could not find powershell"))
    }
}
