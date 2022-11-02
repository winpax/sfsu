use std::{env, path::PathBuf, process::Command};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoopConfig {
    pub last_update: Option<String>,
    pub virustotal_api_key: Option<String>,
    pub scoop_repo: Option<String>,
    pub scoop_branch: Option<String>,
}

impl ScoopConfig {
    pub fn read() -> std::io::Result<Self> {
        let config_path = ScoopConfig::get_path();

        let config = std::fs::read_to_string(config_path)?;

        let config: ScoopConfig = serde_json::from_str(&config)?;

        Ok(config)
    }

    pub fn get_path() -> PathBuf {
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

    pub fn update_last_update_time(&mut self) {
        let date_time = Command::new("powershell.exe")
            .args(["-Command", "[System.DateTime]::Now.ToString('o')"])
            .output()
            .expect("Failed to get time from powershell");

        let stdout_raw = date_time.stdout;
        let stdout = String::from_utf8(stdout_raw).unwrap();

        self.last_update = Some(stdout);
    }

    pub fn save(&self) -> std::io::Result<()> {
        let config_path = ScoopConfig::get_path();

        let config = serde_json::to_string_pretty(self)?;

        std::fs::write(config_path, config)?;

        Ok(())
    }
}
