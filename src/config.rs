use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoopConfig {
    pub last_update: Option<String>,
    pub virustotal_api_key: Option<String>,
    pub scoop_repo: Option<String>,
    pub scoop_branch: Option<String>,
}

pub fn get_config_path() -> PathBuf {
    let xdg_config = env::var("XFG_CONFIG_HOME").map(PathBuf::from);
    let user_profile = env::var("USERPROFILE").map(|path| PathBuf::from(path).join(".config"));

    let path = match (xdg_config, user_profile) {
        (Ok(path), _) => path,
        (_, Ok(path)) => path,
        _ => panic!("Could not find config directory"),
    }
    .join("scoop")
    .join("config.json");

    if !path.exists() {
        panic!("Could not find config file");
    }

    path
}

pub fn read_scoop_config() -> std::io::Result<ScoopConfig> {
    let config_path = get_config_path();

    let config = std::fs::read_to_string(config_path)?;

    let config: ScoopConfig = serde_json::from_str(&config)?;

    Ok(config)
}
