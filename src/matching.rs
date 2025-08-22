use std::str::FromStr;

use glob::Pattern as GlobPattern;
use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse regex pattern: {0}")]
    RegexParse(#[from] regex::Error),
    #[error("Failed to parse glob pattern: {0}")]
    GlobParse(#[from] glob::PatternError),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub enum PatternMatcher {
    Regex(Regex),
    Glob(GlobPattern),
}

#[allow(dead_code)]
impl PatternMatcher {
    pub fn parse_regex(pattern: impl AsRef<str>) -> Result<Self> {
        let regex = Regex::new(pattern.as_ref())?;
        Ok(PatternMatcher::Regex(regex))
    }

    pub fn parse_regex_with(
        prefix: Option<&str>,
        pattern: impl AsRef<str>,
        suffix: Option<&str>,
    ) -> Result<Self> {
        let mut full_pattern = String::with_capacity(
            prefix.map_or(0, str::len) + pattern.as_ref().len() + suffix.map_or(0, str::len),
        );

        if let Some(p) = prefix {
            full_pattern.push_str(p);
        }

        full_pattern.push_str(pattern.as_ref());

        if let Some(s) = suffix {
            full_pattern.push_str(s);
        }

        let regex = Regex::new(&full_pattern)?;
        Ok(PatternMatcher::Regex(regex))
    }

    pub fn parse_glob(pattern: impl AsRef<str>) -> Result<Self> {
        let glob = GlobPattern::new(pattern.as_ref())?;
        Ok(PatternMatcher::Glob(glob))
    }

    pub fn parse_glob_with(
        prefix: Option<&str>,
        pattern: impl AsRef<str>,
        suffix: Option<&str>,
    ) -> Result<Self> {
        let mut full_pattern = String::with_capacity(
            prefix.map_or(0, str::len) + pattern.as_ref().len() + suffix.map_or(0, str::len),
        );

        if let Some(p) = prefix {
            full_pattern.push_str(p);
        }

        full_pattern.push_str(pattern.as_ref());

        if let Some(s) = suffix {
            full_pattern.push_str(s);
        }

        let glob = GlobPattern::new(&full_pattern)?;
        Ok(PatternMatcher::Glob(glob))
    }

    pub fn test(&self, input: &str) -> bool {
        match self {
            PatternMatcher::Regex(regex) => regex.is_match(input),
            PatternMatcher::Glob(glob) => glob.matches(input),
        }
    }
}

#[cfg(not(feature = "v2"))]
impl FromStr for PatternMatcher {
    type Err = anyhow::Error;

    fn from_str(pattern: &str) -> Result<Self, Self::Err> {
        if let Ok(regex) = Self::parse_regex(pattern) {
            Ok(regex)
        } else if let Ok(glob) = Self::parse_glob(pattern) {
            Ok(glob)
        } else {
            Err(anyhow::anyhow!("Invalid pattern"))
        }
    }
}

#[cfg(feature = "v2")]
impl FromStr for PatternMatcher {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(glob) = Self::parse_glob(pattern) {
            Ok(glob)
        } else if let Ok(regex) = Self::parse_regex(pattern) {
            Ok(regex)
        } else {
            Err(anyhow::anyhow!("Invalid pattern"))
        }
    }
}
