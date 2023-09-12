use std::{env, path::PathBuf};

use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scoop {
    // The timestamp of the last scoop update
    pub last_update: Option<String>,
    // The virustotal api key (removed as unused, and shouldn't be read if it doesn't need to be)
    // pub virustotal_api_key: Option<String>,
    pub scoop_repo: Option<String>,
    pub scoop_branch: Option<String>,
    // Scoop path
    pub root_path: Option<String>,
}

impl Scoop {
    /// Converts the config path into the [`Scoop`] struct
    ///
    /// # Errors
    /// - The file was not valid UTF-8
    /// - The read file was did not match the expected structure
    pub fn load() -> std::io::Result<Self> {
        let config_path = Self::get_path();

        let config = std::fs::read_to_string(config_path)?;

        let config: Self = serde_json::from_str(&config)?;

        Ok(config)
    }

    /// Gets the scoop config path
    ///
    /// # Panics
    /// - The config directory does not exist
    pub fn get_path() -> PathBuf {
        let xdg_config = env::var("XFG_CONFIG_HOME").map(PathBuf::from);
        let user_profile = env::var("USERPROFILE")
            .map(PathBuf::from)
            .map(|path| path.join(".config"));

        let path = match (xdg_config, user_profile) {
            (Ok(path), _) | (_, Ok(path)) => path,
            _ => panic!("Could not find config directory"),
        }
        .join("scoop")
        .join("config.json");

        assert!(path.exists(), "Could not find config file");

        path
    }

    // TODO: More solid error type here?
    /// Update the last time the scoop was updated
    ///
    /// # Errors
    /// - If the system time is 262,000 years in the future
    pub fn update_last_update_time(&mut self) -> anyhow::Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};

        use chrono::NaiveDateTime;
        let naive_time = NaiveDateTime::from_timestamp_opt(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs()
                .try_into()?,
            0,
        )
        .ok_or(anyhow::anyhow!(
            "The system time is more than 262,000 years in the future"
        ))?;

        let date_time = naive_time
            .and_local_timezone(Local)
            .earliest()
            .ok_or(anyhow::anyhow!("Local time representation was invalid..."))?;

        self.last_update = Some(date_time.to_string());

        Ok(())
    }

    /// Save the modified scoop config
    ///
    /// # Errors
    /// - The struct could not be serialized to JSON
    /// - The file could not be written
    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::get_path();

        let config = serde_json::to_string_pretty(self)?;

        std::fs::write(config_path, config)?;

        Ok(())
    }
}
