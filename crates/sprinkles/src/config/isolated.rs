//! The path to use for Scoop instead of the system path

use std::ffi::OsString;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum IsolatedPath {
    Bool(bool),
    Path(OsString),
}

impl Default for IsolatedPath {
    fn default() -> Self {
        Self::Bool(false)
    }
}
