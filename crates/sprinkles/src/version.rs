//! Version helpers

use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    num::ParseIntError,
};

#[cfg(feature = "manifest-hashes")]
use getset::Getters;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[cfg(feature = "manifest-hashes")]
use crate::hash::substitutions::SubstitutionMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
/// A struct representing a string version
pub struct Version(String);

impl From<String> for Version {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Version {
    fn part_regex() -> Regex {
        Regex::new(r"[._-]").unwrap()
    }

    /// Create a new version string
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    #[must_use]
    /// Get the version string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    /// Get the version string with dots instead of separators
    pub fn dot_version(&self) -> Cow<'_, str> {
        Self::part_regex().replace_all(&self.0, ".")
    }

    #[must_use]
    /// Get the version string with underscores instead of separators
    pub fn underscore_version(&self) -> Cow<'_, str> {
        Self::part_regex().replace_all(&self.0, "_")
    }

    #[must_use]
    /// Get the version string with dashes instead of separators
    pub fn dash_version(&self) -> Cow<'_, str> {
        Self::part_regex().replace_all(&self.0, "-")
    }

    #[must_use]
    /// Get the version string with all separators removed
    pub fn clean_version(&self) -> Cow<'_, str> {
        Self::part_regex().replace_all(&self.0, "")
    }

    #[cfg(feature = "manifest-hashes")]
    /// Parse the version string into a structured version
    ///
    /// # Errors
    /// Will throw an error if an invalid version string was provided.
    /// This should usually should not panic, and you should just ignore the happy path.
    pub fn parse(&self) -> Result<ParsedVersion, Error> {
        let mut parts = self.0.split('.');

        let major = parts.next().ok_or(Error::MissingFirstPart)?.parse()?;
        let minor = parts.next().and_then(|part| part.parse().ok());
        let patch = parts.next().and_then(|part| part.parse().ok());
        let build = parts.next().map(String::from);
        let pre_release = parts.next().map(String::from);

        Ok(ParsedVersion {
            major,
            minor,
            patch,
            build,
            pre_release,
        })
    }

    #[cfg(feature = "manifest-hashes")]
    #[must_use]
    /// Create a substitution map for the version
    ///
    /// # Panics
    /// - If the hardcoded regex is invalid. (This is a bug, please report it)
    pub fn submap(&self) -> SubstitutionMap {
        let mut map = SubstitutionMap::new();
        map.insert("$version".into(), self.as_str().to_string());
        map.insert("$dotVersion".into(), self.dot_version().to_string());
        map.insert(
            "$underscoreVersion".into(),
            self.underscore_version().to_string(),
        );
        map.insert("$dashVersion".into(), self.dash_version().to_string());
        map.insert("$cleanVersion".into(), self.clean_version().to_string());

        if let Ok(parsed) = self.parse() {
            map.insert("$majorVersion".into(), parsed.major().to_string());

            if let Some(minor) = parsed.minor() {
                map.insert("$minorVersion".into(), minor.to_string());
            }
            if let Some(patch) = parsed.patch() {
                map.insert("$patchVersion".into(), patch.to_string());
            }
            if let Some(build) = parsed.build() {
                map.insert("$buildVersion".into(), build.clone());
            }
            if let Some(pre_release) = parsed.pre_release() {
                map.insert("$preReleaseVersion".into(), pre_release.clone());
            }
        }

        let matches_regex = Regex::new(r"(?<head>\d+\.\d+(?:\.\d+)?)(?<tail>.*)").unwrap();
        if let Some(captures) = matches_regex.captures(self.as_str()) {
            // The following two `if let` statements in theory should always be true
            // But to avoid a panic in case of a bug, we are using `if let` instead of `unwrap`

            if let Some(head) = captures.name("head") {
                map.insert("$matchHead".into(), head.as_str().to_string());
            }

            if let Some(tail) = captures.name("tail") {
                map.insert("$matchTail".into(), tail.as_str().to_string());
            }
        }

        // TODO: Add custom matches for version

        map
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&Version> for semver::Version {
    type Error = semver::Error;

    fn try_from(value: &Version) -> Result<Self, Self::Error> {
        value.0.parse()
    }
}

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Errors that can occur when parsing a version string
pub enum Error {
    #[error("Failed to parse integer: {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("The version string is missing the first part. Likely an empty string")]
    MissingFirstPart,
}

#[cfg(feature = "manifest-hashes")]
#[derive(Debug, Clone, Getters)]
#[get = "pub"]
/// A structured version
pub struct ParsedVersion {
    /// Major version
    major: u64,
    /// Minor version
    minor: Option<u64>,
    /// Patch version
    patch: Option<u64>,
    /// Build version
    build: Option<String>,
    /// Pre-release version
    pre_release: Option<String>,
}

#[cfg(feature = "manifest-hashes")]
impl ParsedVersion {
    #[must_use]
    /// Get the version as a simple [`Version`] string
    pub fn to_unparsed(&self) -> Version {
        Version(self.to_string())
    }
}

#[cfg(feature = "manifest-hashes")]
impl Display for ParsedVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;

        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }

        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }

        if let Some(build) = &self.build {
            write!(f, ".{build}")?;
        }

        if let Some(pre_release) = &self.pre_release {
            write!(f, "-{pre_release}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    #[test]
    fn test_version() {
        let v1 = Version::new(1, 2, 3);
        assert_eq!(v1.to_string(), "1.2.3");

        let v3 = Version::new(3, 2, 1);
        assert_eq!(v3.to_string(), "3.2.1");

        let v2 = Version::new(2, 1, 3);

        assert!(v1 < v3);
        assert!(v1 < v2);

        assert_eq!(v1, Version::new(1, 2, 3));
    }
}
