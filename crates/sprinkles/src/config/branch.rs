//! Scoop branch configuration

use std::fmt::Display;

use serde::{Deserialize, Serialize};

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
