//! Scoop branch configuration

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// Which Scoop branch to use
pub enum ScoopBranch {
    #[default]
    /// The default branch
    Master,
    /// The develop branch
    Develop,
    /// A custom branch
    Other(String),
}

impl Serialize for ScoopBranch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for ScoopBranch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "master" => Ok(Self::Master),
            "develop" => Ok(Self::Develop),
            _ => Ok(Self::Other(s)),
        }
    }
}

impl ScoopBranch {
    #[must_use]
    /// Get the branch name
    pub fn name(&self) -> &str {
        match self {
            ScoopBranch::Master => "master",
            ScoopBranch::Develop => "develop",
            Self::Other(branch) => branch.as_str(),
        }
    }
}

impl Display for ScoopBranch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
