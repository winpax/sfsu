//! Scoop config helpers

use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Scoop configuration
pub struct Scoop {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The timestamp of the last scoop update
    pub last_update: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The virustotal api key
    pub virustotal_api_key: Option<String>,

    /// The bucket to use for the scoop
    pub scoop_repo: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The branch to use for the scoop
    pub scoop_branch: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// Scoop path
    pub root_path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// The cache path
    pub cache_path: Option<PathBuf>,

    #[serde(flatten)]
    /// Any other values in the config
    other: Map<String, Value>,
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

    /// Update the last time the scoop was updated
    pub fn update_last_update_time(&mut self) {
        let date_time = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, false);

        self.last_update = Some(date_time.to_string());
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
