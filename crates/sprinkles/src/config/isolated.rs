//! The path to use for Scoop instead of the system path

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum IsolatedPath {
    Bool(bool),
    Path(String),
}

impl Default for IsolatedPath {
    fn default() -> Self {
        Self::Bool(false)
    }
}
