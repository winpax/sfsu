//! Scoop shim handler

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
