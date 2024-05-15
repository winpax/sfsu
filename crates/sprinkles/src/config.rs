//! Scoop config helpers

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{proxy::Proxy, Architecture};

pub mod aria2;
pub mod branch;
pub mod repo;
pub mod shim;

mod defaults;
mod isolated;
mod skips;

use skips::Skip;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
/// Scoop configuration
pub struct Scoop {
    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// External 7zip (from path) will be used for archives extraction
    pub use_external_7zip: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Prefer lessmsi utility over native msiexec
    pub use_lessmsi: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// The 'current' version alias will not be used. Shims and shortcuts will point to specific version instead
    pub no_junction: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Git repository containing the scoop adaptor's source code
    ///
    /// This configuration is useful for custom forks of scoop, or a scoop replacement
    pub scoop_repo: repo::ScoopRepo,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Allow to use different branch than master
    ///
    /// Could be used for testing specific functionalities before released into all users
    ///
    /// If you want to receive updates earlier to test new functionalities use develop (see: 'https://github.com/ScoopInstaller/Scoop/issues/2939')
    pub scoop_branch: branch::ScoopBranch,

    /// By default, we will use the proxy settings from Internet Options, but with anonymous authentication.
    ///
    ///   * To use the credentials for the current logged-in user, use 'currentuser' in place of username:password
    ///   * To use the system proxy settings configured in Internet Options, use 'default' in place of host:port
    ///   * An empty or unset value for proxy is equivalent to 'default' (with no username or password)
    ///   * To bypass the system proxy and connect directly, use 'none' (with no username or password)
    pub proxy: Option<Proxy>,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// When a conflict is detected during updating, Scoop will auto-stash the uncommitted changes.
    /// (Default is `false`, which will abort the update)
    pub autostash_on_conflict: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Allow to configure preferred architecture for application installation
    ///
    /// If not specified, architecture is determined by system
    pub default_architecture: Architecture,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Additional and detailed output will be shown
    pub debug: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Force apps updating to bucket's version
    pub force_update: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Show update log
    pub show_update_log: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Displays the manifest of every app that's about to
    /// be installed, then asks user if they wish to proceed
    pub show_manifest: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Choose scoop shim build
    pub shim: shim::ScoopShim,

    #[serde(
        default = "defaults::default_scoop_root_path",
        deserialize_with = "defaults::deserialize_scoop_root_path"
    )]
    /// Path to Scoop root directory
    pub root_path: PathBuf,

    #[serde(default = "defaults::default_scoop_global_path")]
    /// Path to Scoop root directory for global apps
    pub global_path: PathBuf,

    /// For downloads, defaults to 'cache' folder under Scoop root directory
    pub cache_path: Option<PathBuf>,

    /// GitHub API token used to make authenticated requests
    ///
    /// This is essential for checkver and similar functions to run without
    /// incurring rate limits and download from private repositories
    pub gh_token: Option<String>,

    /// API key used for uploading/scanning files using virustotal
    ///
    /// See: 'https://support.virustotal.com/hc/en-us/articles/115002088769-Please-give-me-an-API-key'
    pub virustotal_api_key: Option<String>,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// When set to `false` (default), Scoop would stop its procedure immediately if it detects
    /// any target app process is running. Procedure here refers to reset/uninstall/update.
    ///
    /// When set to `true`, Scoop only displays a warning message and continues procedure.
    pub ignore_running_processes: bool,

    /// Disable/Hold Scoop self-updates, until the specified date
    /// `scoop hold scoop` will set the value to one day later
    ///
    /// Should be in the format 'YYYY-MM-DD', 'YYYY/MM/DD' or any other forms that accepted by '[System.DateTime]::Parse()'
    ///
    /// Ref: https://docs.microsoft.com/dotnet/api/system.datetime.parse?view=netframework-4.5
    pub hold_update_until: Option<String>,

    #[serde(default)]
    /// When set to `true` (default), Scoop will use `SCOOP_PATH` environment variable to store apps' `PATH`s.
    ///
    /// When set to arbitrary non-empty string, Scoop will use that string as the environment variable name instead.
    /// This is useful when you want to isolate Scoop from the system `PATH`.
    pub use_isolated_path: isolated::IsolatedPath,

    /// The timestamp of the last scoop update
    pub last_update: Option<String>,

    #[serde(flatten)]
    /// Aria2 configuration
    pub aria2_config: aria2::Config,

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

    #[must_use]
    /// Gets the scoop config path
    ///
    /// # Panics
    /// - The config directory does not exist
    pub fn get_path() -> PathBuf {
        let config_dir = crate::env::paths::config_dir();

        let path = config_dir
            .expect("Could not find config directory")
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

    /// Make the config strict
    ///
    /// This will remove all fields that are not in the config struct
    pub fn make_strict(&mut self) {
        self.other = Map::new();
    }

    /// Convert the config to a JSON object
    ///
    /// # Errors
    /// - The config could not be converted to a JSON object
    pub fn to_object(&self) -> serde_json::Result<Value> {
        serde_json::to_value(self)
    }

    /// Convert the JSON object to [`Scoop`] config
    ///
    /// # Errors
    /// - The JSON object could not be deserialized to a [`Scoop`] config
    pub fn from_object(object: Value) -> serde_json::Result<Self> {
        serde_json::from_value(object)
    }
}

#[cfg(test)]
mod tests {
    use tests::isolated::IsolatedPath;

    use super::*;

    #[test]
    fn test_isolated_path() {
        let path = IsolatedPath::Path("NOT_SCOOP_PATH".into());
        let serialized = serde_json::to_string(&path).unwrap();
        let deserialized: IsolatedPath = serde_json::from_str(serialized.as_str()).unwrap();

        assert_eq!(path, deserialized);

        let path = "true";
        let deserialized: IsolatedPath = serde_json::from_str(path).unwrap();

        assert_eq!(IsolatedPath::Bool(true), deserialized);

        let path = "false";
        let deserialized: IsolatedPath = serde_json::from_str(path).unwrap();

        assert_eq!(IsolatedPath::Bool(false), deserialized);
    }
}

// Scoop config output:
//
// use_external_7zip: $true|$false
//       External 7zip (from path) will be used for archives extraction.
//
// use_lessmsi: $true|$false
//       Prefer lessmsi utility over native msiexec.
//
// use_sqlite_cache: $true|$false
//       Use SQLite database for caching. This is useful for speeding up 'scoop search' and 'scoop shim' commands.
//
// no_junction: $true|$false
//       The 'current' version alias will not be used. Shims and shortcuts will point to specific version instead.
//
// scoop_repo: http://github.com/ScoopInstaller/Scoop
//       Git repository containining scoop source code.
//       This configuration is useful for custom forks.
//
// scoop_branch: master|develop
//       Allow to use different branch than master.
//       Could be used for testing specific functionalities before released into all users.
//       If you want to receive updates earlier to test new functionalities use develop (see: 'https://github.com/ScoopInstaller/Scoop/issues/2939')
//
// proxy: [username:password@]host:port
//       By default, Scoop will use the proxy settings from Internet Options, but with anonymous authentication.
//
//       * To use the credentials for the current logged-in user, use 'currentuser' in place of username:password
//       * To use the system proxy settings configured in Internet Options, use 'default' in place of host:port
//       * An empty or unset value for proxy is equivalent to 'default' (with no username or password)
//       * To bypass the system proxy and connect directly, use 'none' (with no username or password)
//
// autostash_on_conflict: $true|$false
//       When a conflict is detected during updating, Scoop will auto-stash the uncommitted changes.
//       (Default is $false, which will abort the update)
//
// default_architecture: 64bit|32bit|arm64
//       Allow to configure preferred architecture for application installation.
//       If not specified, architecture is determined by system.
//
// debug: $true|$false
//       Additional and detailed output will be shown.
//
// force_update: $true|$false
//       Force apps updating to bucket's version.
//
// show_update_log: $true|$false
//       Do not show changed commits on 'scoop update'
//
// show_manifest: $true|$false
//       Displays the manifest of every app that's about to
//       be installed, then asks user if they wish to proceed.
//
// shim: kiennq|scoopcs|71
//       Choose scoop shim build.
//
// root_path: $Env:UserProfile\scoop
//       Path to Scoop root directory.
//
// global_path: $Env:ProgramData\scoop
//       Path to Scoop root directory for global apps.
//
// cache_path:
//       For downloads, defaults to 'cache' folder under Scoop root directory.
//
// gh_token:
//       GitHub API token used to make authenticated requests.
//       This is essential for checkver and similar functions to run without
//       incurring rate limits and download from private repositories.
//
// virustotal_api_key:
//       API key used for uploading/scanning files using virustotal.
//       See: 'https://support.virustotal.com/hc/en-us/articles/115002088769-Please-give-me-an-API-key'
//
// cat_style:
//       When set to a non-empty string, Scoop will use 'bat' to display the manifest for
//       the `scoop cat` command and while doing manifest review. This requires 'bat' to be
//       installed (run `scoop install bat` to install it), otherwise errors will be thrown.
//       The accepted values are the same as ones passed to the --style flag of 'bat'.
//
// ignore_running_processes: $true|$false
//       When set to $false (default), Scoop would stop its procedure immediately if it detects
//       any target app process is running. Procedure here refers to reset/uninstall/update.
//       When set to $true, Scoop only displays a warning message and continues procedure.
//
// private_hosts:
//       Array of private hosts that need additional authentication.
//       For example, if you want to access a private GitHub repository,
//       you need to add the host to this list with 'match' and 'headers' strings.
//
// hold_update_until:
//       Disable/Hold Scoop self-updates, until the specified date.
//       `scoop hold scoop` will set the value to one day later.
//       Should be in the format 'YYYY-MM-DD', 'YYYY/MM/DD' or any other forms that accepted by '[System.DateTime]::Parse()'.
//       Ref: https://docs.microsoft.com/dotnet/api/system.datetime.parse?view=netframework-4.5#StringToParse
//
// update_nightly: $true|$false
//       Nightly version is formatted as 'nightly-yyyyMMdd' and will be updated after one day if this is set to $true.
//       Otherwise, nightly version will not be updated unless `--force` is used.
//
// use_isolated_path: $true|$false|[string]
//       When set to $true, Scoop will use `SCOOP_PATH` environment variable to store apps' `PATH`s.
//       When set to arbitrary non-empty string, Scoop will use that string as the environment variable name instead.
//       This is useful when you want to isolate Scoop from the system `PATH`.
