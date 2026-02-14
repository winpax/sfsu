use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::system::paths::Paths;

// pub fn default_scoop_repo() -> String {
//     "https://github.com/ScoopInstaller/Scoop".into()
// }

pub fn default_scoop_root_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("home dir");
    path.push("scoop");
    path
}

pub fn deserialize_scoop_root_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_else(default_scoop_root_path))
}

/// Gets the default scoop path
///
/// Note, we do not create the directory here,
/// as it causes too many issues when not running as admin
///
/// This should be handled manually by implementations, when running as admin
pub fn default_scoop_global_path() -> PathBuf {
    Paths::CommonAppData
        .into_path()
        .unwrap_or_else(|| "C:\\ProgramData".into())
        .join("scoop")
}
