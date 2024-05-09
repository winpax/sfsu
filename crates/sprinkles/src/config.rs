//! Scoop config helpers

use std::{env, fmt::Display, path::PathBuf};

use serde::{de::Visitor, Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;

use crate::{proxy::Proxy, Architecture};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Which Scoop branch to use
pub enum ScoopBranch {
    #[default]
    /// The default branch
    Master,
    /// The develop branch
    Develop,
}

impl ScoopBranch {
    #[must_use]
    /// Get the branch name
    pub fn name(self) -> &'static str {
        match self {
            ScoopBranch::Master => "master",
            ScoopBranch::Develop => "develop",
        }
    }
}

impl Display for ScoopBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Scoop shim builds
pub enum ScoopShim {
    #[default]
    /// Use the kiennq shim
    Kiennq,
    /// Use the scoopcs shim
    Scoopcs,
    #[serde(rename = "71")]
    /// Use the 71 shim
    SeventyOne,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// The git repository containing the scoop adaptor's source code
pub struct ScoopRepo(String);

impl ScoopRepo {
    #[must_use]
    /// Get the url to the git repository containing the scoop adaptor's source code
    pub fn url(&self) -> &str {
        &self.0
    }
}

impl Default for ScoopRepo {
    fn default() -> Self {
        Self("https://github.com/ScoopInstaller/Scoop".into())
    }
}

impl Display for ScoopRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// The path to Scoop root directory
pub struct ScoopRootPath(PathBuf);

impl Default for ScoopRootPath {
    fn default() -> Self {
        Self({
            let mut path = PathBuf::from(env::var("USERPROFILE").unwrap());
            path.push("scoop");
            path
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// The path to Scoop root directory for global apps
pub struct ScoopGlobalPath(PathBuf);

impl Default for ScoopGlobalPath {
    fn default() -> Self {
        Self({
            let mut path = PathBuf::from(env::var("ProgramData").unwrap());
            path.push("scoop");
            path
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// The path to use for Scoop instead of the system path
pub struct IsolatedPath(PathBuf);

impl Serialize for IsolatedPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let path = self.0.as_path();
        if let Ok(scoop_path_env) = env::var("SCOOP_PATH") {
            if path == PathBuf::from(scoop_path_env) {
                return serializer.serialize_bool(true);
            }
        }

        serializer.serialize_str(&path.display().to_string())
    }
}

struct IsolatedPathVisitor;
impl<'de> Visitor<'de> for IsolatedPathVisitor {
    type Value = IsolatedPath;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("true or a custom variable name")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v {
            Ok(IsolatedPath(PathBuf::from(env::var("SCOOP_PATH").unwrap())))
        } else {
            Err(serde::de::Error::custom("expected true"))
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "true" => self.visit_bool(true),
            "false" => self.visit_bool(false),
            _ => Ok(IsolatedPath(PathBuf::from(v))),
        }
    }
}

impl<'de> Deserialize<'de> for IsolatedPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IsolatedPathVisitor)
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
/// Scoop configuration
pub struct Scoop {
    #[serde(default)]
    /// External 7zip (from path) will be used for archives extraction
    pub use_external_7zip: bool,

    #[serde(default)]
    /// Prefer lessmsi utility over native msiexec
    pub use_lessmsi: bool,

    #[serde(default)]
    /// The 'current' version alias will not be used. Shims and shortcuts will point to specific version instead
    pub no_junction: bool,

    /// Git repository containing the scoop adaptor's source code
    ///
    /// This configuration is useful for custom forks of scoop, or a scoop replacement
    pub scoop_repo: ScoopRepo,

    #[serde(default)]
    /// Allow to use different branch than master
    ///
    /// Could be used for testing specific functionalities before released into all users
    ///
    /// If you want to receive updates earlier to test new functionalities use develop (see: 'https://github.com/ScoopInstaller/Scoop/issues/2939')
    pub scoop_branch: ScoopBranch,

    /// By default, we will use the proxy settings from Internet Options, but with anonymous authentication.
    ///
    ///   * To use the credentials for the current logged-in user, use 'currentuser' in place of username:password
    ///   * To use the system proxy settings configured in Internet Options, use 'default' in place of host:port
    ///   * An empty or unset value for proxy is equivalent to 'default' (with no username or password)
    ///   * To bypass the system proxy and connect directly, use 'none' (with no username or password)
    pub proxy: Option<Proxy>,

    #[serde(default)]
    /// When a conflict is detected during updating, Scoop will auto-stash the uncommitted changes.
    /// (Default is `false`, which will abort the update)
    pub autostash_on_conflict: bool,

    #[serde(default)]
    /// Allow to configure preferred architecture for application installation
    ///
    /// If not specified, architecture is determined by system
    pub default_architecture: Architecture,

    #[serde(default)]
    /// Additional and detailed output will be shown
    pub debug: bool,

    #[serde(default)]
    /// Force apps updating to bucket's version
    pub force_update: bool,

    #[serde(default)]
    /// Show update log
    pub show_update_log: bool,

    #[serde(default)]
    /// Displays the manifest of every app that's about to
    /// be installed, then asks user if they wish to proceed
    pub show_manifest: bool,

    #[serde(default)]
    /// Choose scoop shim build
    pub shim: ScoopShim,

    #[serde(default = "defaults::default_scoop_root_path")]
    /// Path to Scoop root directory
    pub root_path: PathBuf,

    #[serde(default = "defaults::default_scoop_global_path")]
    /// Path to Scoop root directory for global apps
    pub global_path: PathBuf,

    #[serde(default = "defaults::default_cache_path")]
    /// For downloads, defaults to 'cache' folder under Scoop root directory
    pub cache_path: PathBuf,

    /// GitHub API token used to make authenticated requests
    ///
    /// This is essential for checkver and similar functions to run without
    /// incurring rate limits and download from private repositories
    pub gh_token: Option<String>,

    /// API key used for uploading/scanning files using virustotal
    ///
    /// See: 'https://support.virustotal.com/hc/en-us/articles/115002088769-Please-give-me-an-API-key'
    pub virustotal_api_key: Option<String>,

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

    /// When set to `true` (default), Scoop will use `SCOOP_PATH` environment variable to store apps' `PATH`s.
    ///
    /// When set to arbitrary non-empty string, Scoop will use that string as the environment variable name instead.
    /// This is useful when you want to isolate Scoop from the system `PATH`.
    pub use_isolated_path: Option<IsolatedPath>,

    /// The timestamp of the last scoop update
    pub(crate) last_update: Option<String>,

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
}

mod defaults {
    use std::{env, path::PathBuf};

    use crate::contexts::{ScoopContext, User};

    pub(super) fn default_scoop_root_path() -> PathBuf {
        let mut path = PathBuf::from(env::var("USERPROFILE").unwrap());
        path.push("scoop");
        path
    }

    pub(super) fn default_scoop_global_path() -> PathBuf {
        use std::{ffi::OsString, os::windows::ffi::OsStringExt};

        use windows::Win32::{
            Foundation::{HWND, MAX_PATH},
            UI::Shell::{SHGetSpecialFolderPathW, CSIDL_COMMON_APPDATA},
        };

        let mut buf = [0u16; MAX_PATH as usize];
        let success = unsafe {
            #[allow(clippy::cast_possible_wrap)]
            SHGetSpecialFolderPathW(HWND::default(), &mut buf, CSIDL_COMMON_APPDATA as i32, true)
                .as_bool()
        };

        let path = if success {
            let string = OsString::from_wide(&buf);
            let utf8_string = string.to_string_lossy();
            let trimmed = utf8_string.trim_end_matches('\0');

            PathBuf::from(trimmed)
        } else {
            "C:\\ProgramData".into()
        }
        .join("scoop");

        if !path.exists() {
            std::fs::create_dir(&path).expect("could not create scoop global path");
        }

        path
    }

    pub(super) fn default_cache_path() -> PathBuf {
        User::new().sub_path("cache")
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use super::IsolatedPath;

    #[test]
    fn test_isolated_path_serde() {
        let true_path = IsolatedPath(
            env::var("SCOOP_PATH")
                .map(PathBuf::from)
                .unwrap_or_default(),
        );

        let custom_path = IsolatedPath(PathBuf::from("custom"));

        let true_path_ser = serde_json::to_string(&true_path).unwrap();
        let custom_path_ser = serde_json::to_string(&custom_path).unwrap();

        let true_path_desere: IsolatedPath = serde_json::from_str(&true_path_ser).unwrap();
        let custom_path_desere: IsolatedPath = serde_json::from_str(&custom_path_ser).unwrap();

        assert_eq!(true_path, true_path_desere);
        assert_eq!(custom_path, custom_path_desere);
    }
}
