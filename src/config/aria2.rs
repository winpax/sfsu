//! Aria2 configuration

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::Skip;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
/// Scoop's Aria2 configuration
pub struct Config {
    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Aria2c will be used for downloading of artifacts.
    pub aria2_enabled: bool,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Disable Aria2c warning which is shown while downloading.
    pub aria2_warning_enabled: bool,

    #[serde(
        default = "default_retry_wait",
        skip_serializing_if = "skip_retry_wait"
    )]
    /// Number of seconds to wait between retries.
    ///
    /// See: <https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-retry-wait>
    pub aria2_retry_wait: u64,

    #[serde(default = "default_split", skip_serializing_if = "skip_split")]
    /// Number of connections used for downlaod.
    ///
    /// See: <https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-s>
    pub aria2_split: u64,

    #[serde(
        default = "default_max_connection_per_server",
        skip_serializing_if = "skip_max_connection_per_server"
    )]
    /// The maximum number of connections to one server for each download.
    ///
    /// See: <https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-x>
    pub aria2_max_connection_per_server: u64,

    #[serde(
        default = "default_min_split_size",
        skip_serializing_if = "skip_min_split_size"
    )]
    /// Downloaded files will be splitted by this configured size and downloaded using multiple connections.
    ///
    /// See: <https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-k>
    pub aria2_min_split_size: String,

    #[serde(default, skip_serializing_if = "Skip::skip")]
    /// Array of additional aria2 options.
    ///
    /// See: <https://aria2.github.io/manual/en/html/aria2c.html#options>
    pub aria2_options: Vec<String>,
}

fn default_retry_wait() -> u64 {
    2
}
#[allow(clippy::trivially_copy_pass_by_ref)]
fn skip_retry_wait(value: &u64) -> bool {
    *value == default_retry_wait()
}

fn default_split() -> u64 {
    5
}
#[allow(clippy::trivially_copy_pass_by_ref)]
fn skip_split(value: &u64) -> bool {
    *value == default_split()
}

fn default_max_connection_per_server() -> u64 {
    5
}
#[allow(clippy::trivially_copy_pass_by_ref)]
fn skip_max_connection_per_server(value: &u64) -> bool {
    *value == default_max_connection_per_server()
}

fn default_min_split_size() -> String {
    "5M".to_string()
}
fn skip_min_split_size(value: &String) -> bool {
    value == &default_min_split_size()
}

// Scoop config output:
//
// aria2-enabled: $true|$false
//       Aria2c will be used for downloading of artifacts.
//
// aria2-warning-enabled: $true|$false
//       Disable Aria2c warning which is shown while downloading.
//
// aria2-retry-wait: 2
//       Number of seconds to wait between retries.
//       See: 'https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-retry-wait'
//
// aria2-split: 5
//       Number of connections used for downlaod.
//       See: 'https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-s'
//
// aria2-max-connection-per-server: 5
//       The maximum number of connections to one server for each download.
//       See: 'https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-x'
//
// aria2-min-split-size: 5M
//       Downloaded files will be splitted by this configured size and downloaded using multiple connections.
//       See: 'https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-k'
//
// aria2-options:
//       Array of additional aria2 options.
//       See: 'https://aria2.github.io/manual/en/html/aria2c.html#options'
