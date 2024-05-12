//! Scoop repository handler

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
/// The git repository containing the scoop adaptor's source code
pub struct ScoopRepo(String);

impl ScoopRepo {
    #[must_use]
    /// Get the url to the git repository containing the scoop adaptor's source code
    pub fn url(&self) -> &str {
        &self.0
    }

    #[must_use]
    /// Get the url to the git repository containing the scoop adaptor's source code
    pub fn as_str(&self) -> &str {
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

impl From<&str> for ScoopRepo {
    fn from(url: &str) -> Self {
        Self(url.to_string())
    }
}

impl From<String> for ScoopRepo {
    fn from(url: String) -> Self {
        Self(url)
    }
}

impl From<ScoopRepo> for String {
    fn from(repo: ScoopRepo) -> Self {
        repo.0
    }
}
