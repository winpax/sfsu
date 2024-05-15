//! The path to use for Scoop instead of the system path

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IsolatedPath {
    Bool(bool),
    Path(PathBuf),
}

impl Default for IsolatedPath {
    fn default() -> Self {
        Self::Bool(false)
    }
}
