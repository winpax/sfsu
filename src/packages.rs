use std::{
    path::Path,
    time::{SystemTimeError, UNIX_EPOCH},
};

use chrono::NaiveDateTime;
use clap::{Parser, ValueEnum};
use colored::Colorize as _;
use itertools::Itertools as _;
use quork::traits::truthy::ContainsTruth as _;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    buckets::{self, Bucket},
    output::{
        sectioned::{Children, Section, Text},
        wrappers::time::NicerNaiveTime,
    },
    Scoop,
};

pub mod install;
pub mod manifest;
pub mod reference;

pub use install::Manifest as InstallManifest;
pub use manifest::Manifest;

use manifest::StringOrArrayOfStringsOrAnArrayOfArrayOfStrings;

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MinInfo {
    pub name: String,
    pub version: String,
    pub source: String,
    pub updated: NicerNaiveTime,
    pub notes: String,
}

impl MinInfo {
    /// Parse minmal package info for every installed app
    ///
    /// # Errors
    /// - Invalid file names
    /// - File metadata errors
    /// - Invalid time
    pub fn list_installed(bucket: Option<&String>) -> Result<Vec<Self>> {
        let apps = Scoop::installed_apps()?;

        apps.par_iter()
            .map(Self::from_path)
            .filter(|package| {
                if let Ok(pkg) = package {
                    if let Some(bucket) = bucket {
                        return &pkg.source == bucket;
                    }
                }
                // Keep errors so that the following line will return the error
                true
            })
            .collect()
    }

    /// Parse minimal package into from a given path
    ///
    /// # Errors
    /// - Invalid file names
    /// - File metadata errors
    /// - Invalid time
    ///
    /// # Panics
    /// - Date time invalid or out of range
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let package_name = path
            .file_name()
            .map(|f| f.to_string_lossy())
            .ok_or(PackageError::MissingFileName)?;

        let naive_time = {
            let updated = {
                let updated_sys = path.metadata()?.modified()?;

                updated_sys.duration_since(UNIX_EPOCH)?.as_secs()
            };

            #[allow(clippy::cast_possible_wrap)]
            NaiveDateTime::from_timestamp_opt(updated as i64, 0)
                .expect("invalid or out-of-range datetime")
        };

        let app_current = path.join("current");

        let manifest = Manifest::from_path(app_current.join("manifest.json")).unwrap_or_default();

        let install_manifest =
            InstallManifest::from_path(app_current.join("install.json")).unwrap_or_default();

        Ok(Self {
            name: package_name.to_string(),
            version: manifest.version,
            source: install_manifest.get_source(),
            updated: naive_time.into(),
            notes: if install_manifest.hold.contains_truth() {
                String::from("Held")
            } else {
                String::new()
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("Invalid utf8 found. This is not supported by sfsu")]
    NonUtf8,
    #[error("Missing or invalid file name. The path terminated in '..' or wasn't valid utf8")]
    MissingFileName,
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Could not parse manifest \"{0}\". Failed with error: {1}")]
    ParsingManifest(String, serde_json::Error),
    #[error("Interacting with buckets: {0}")]
    BucketError(#[from] buckets::BucketError),
    #[error("System Time: {0}")]
    TimeError(#[from] SystemTimeError),
}

#[derive(Debug, Default, Copy, Clone, ValueEnum, Display, Parser, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum SearchMode {
    #[default]
    Name,
    Binary,
    Both,
}

impl SearchMode {
    #[must_use]
    pub fn match_names(self) -> bool {
        matches!(self, SearchMode::Name | SearchMode::Both)
    }

    #[must_use]
    pub fn only_match_names(self) -> bool {
        self == SearchMode::Name
    }

    #[must_use]
    pub fn match_binaries(self) -> bool {
        matches!(self, SearchMode::Binary | SearchMode::Both)
    }

    #[must_use]
    pub fn only_match_binaries(self) -> bool {
        self == SearchMode::Binary
    }

    #[must_use]
    pub fn eager_name_matches(self, manifest_name: &str, search_regex: &Regex) -> bool {
        if self.only_match_names() && search_regex.is_match(manifest_name) {
            return true;
        }
        if self.match_binaries() {
            return true;
        }

        false
    }
}

#[derive(Debug, Clone)]
#[must_use = "MatchCriteria has no side effects"]
pub struct MatchCriteria {
    name: bool,
    bins: Vec<String>,
}

impl MatchCriteria {
    pub const fn new() -> Self {
        Self {
            name: false,
            bins: vec![],
        }
    }

    pub fn matches(
        file_name: &str,
        manifest: Option<&Manifest>,
        mode: SearchMode,
        pattern: &Regex,
    ) -> Self {
        let file_name = file_name.to_string();

        let mut output = MatchCriteria::new();

        if mode.match_names() && pattern.is_match(&file_name) {
            output.name = true;
        }

        if let Some(manifest) = manifest {
            let binaries = manifest
                .bin
                .clone()
                .map(|b| b.into_vec())
                .unwrap_or_default();

            let binary_matches = binaries
                .into_iter()
                .filter(|binary| pattern.is_match(binary))
                .filter_map(|b| {
                    if pattern.is_match(&b) {
                        Some(b.clone())
                    } else {
                        None
                    }
                });

            output.bins.extend(binary_matches);
        }

        output
    }
}

pub type Result<T> = std::result::Result<T, PackageError>;

pub trait CreateManifest
where
    Self: Default + for<'a> Deserialize<'a>,
{
    /// Convert a path into a manifest
    ///
    /// # Errors
    /// - The file does not exist
    /// - The file was not valid UTF-8
    fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)?;

        Self::from_str(contents)
            // TODO: Maybe figure out a better approach to this, but it works for now
            .map(|s| s.with_name(path))
            .map_err(|e| PackageError::ParsingManifest(path.display().to_string(), e))
    }

    /// # Errors
    /// - The contents are not a valid manifest
    fn from_str(contents: String) -> serde_json::Result<Self> {
        let trimmed = contents.trim_start_matches('\u{feff}');

        serde_json::from_str(trimmed)
    }

    #[must_use]
    fn with_name(self, path: &Path) -> Self;
}

impl CreateManifest for Manifest {
    fn with_name(mut self, path: &Path) -> Self {
        self.name = path
            .with_extension("")
            .file_name()
            .map(|f| f.to_string_lossy())
            .expect("File to have file name")
            .to_string();

        self
    }
}

impl CreateManifest for InstallManifest {
    fn with_name(mut self, path: &Path) -> Self {
        self.name = path
            .with_extension("")
            .file_name()
            .map(|f| f.to_string_lossy())
            .expect("File to have name")
            .to_string();

        self
    }
}

impl InstallManifest {
    /// List all install manifests
    ///
    /// # Errors
    /// - Invalid install manifest
    /// - Reading directories fails
    pub fn list_all() -> Result<Vec<Self>> {
        Scoop::installed_apps()?
            .par_iter()
            .map(|path| Self::from_path(path.join("current/install.json")))
            .collect::<Result<Vec<_>>>()
    }

    /// List all install manifests, ignoring errors
    ///
    /// # Errors
    /// - Reading directories fails
    pub fn list_all_unchecked() -> Result<Vec<Self>> {
        Ok(Scoop::installed_apps()?
            .par_iter()
            .filter_map(
                |path| match Self::from_path(path.join("current/install.json")) {
                    Ok(v) => Some(v.with_name(path)),
                    Err(_) => None,
                },
            )
            .collect::<Vec<_>>())
    }
}

impl Manifest {
    #[must_use]
    pub fn with_bucket(mut self, bucket: &Bucket) -> Self {
        self.bucket = bucket.name().to_string();

        self
    }

    #[must_use]
    /// List the dependencies of a given manifest, in the order that they will be installed
    ///
    /// Note that this does not include the package itself as a dependency
    pub fn depends(&self) -> Vec<reference::Package> {
        self.depends
            .clone()
            .map(manifest::TOrArrayOfTs::into_vec)
            .unwrap_or_default()
    }

    /// Gets the manifest from a bucket and manifest name
    ///
    /// # Errors
    /// - If the manifest doesn't exist or bucket is invalid
    pub fn from_reference((bucket, name): (String, String)) -> Result<Self> {
        Bucket::new(bucket)?.get_manifest(name)
    }

    #[must_use]
    pub fn binary_matches(&self, regex: &Regex) -> Option<Vec<String>> {
        match self.bin {
            Some(StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::String(ref binary)) => {
                if regex.is_match(binary) {
                    Some(vec![binary.clone()])
                } else {
                    None
                }
            }
            Some(StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::StringArray(ref binaries)) => {
                let matched: Vec<_> = binaries
                    .iter()
                    .filter(|binary| regex.is_match(binary))
                    .cloned()
                    .collect();

                if matched.is_empty() {
                    None
                } else {
                    Some(matched)
                }
            }
            _ => None,
        }
    }

    /// List all installed app manifests
    ///
    /// # Errors
    /// - Invalid install manifest
    /// - Reading directories fails
    ///
    /// # Panics
    /// - If the file name is invalid
    pub fn list_installed() -> Result<Vec<Result<Self>>> {
        Ok(Scoop::installed_apps()?
            .par_iter()
            .map(|path| {
                Self::from_path(path.join("current/manifest.json")).and_then(|mut manifest| {
                    manifest.name = path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .ok_or(PackageError::MissingFileName)?;

                    Ok(manifest)
                })
            })
            .collect::<Vec<_>>())
    }

    pub fn parse_output(
        &self,
        bucket: impl AsRef<str>,
        installed_only: bool,
        pattern: &Regex,
        mode: SearchMode,
    ) -> Option<Section<Text<String>>> {
        // TODO: Better display of output

        // This may be a bit of a hack, but it works

        let match_output = MatchCriteria::matches(
            &self.name,
            if mode.match_binaries() {
                Some(self)
            } else {
                None
            },
            mode,
            pattern,
        );

        if !match_output.name && match_output.bins.is_empty() {
            return None;
        }

        let is_installed = is_installed(&self.name, Some(bucket));
        if installed_only && !is_installed {
            return None;
        }

        let styled_package_name = if self.name == pattern.to_string() {
            self.name.bold().to_string()
        } else {
            self.name.clone()
        };

        let installed_text = if is_installed && !installed_only {
            "[installed] "
        } else {
            ""
        };

        let title = format!("{styled_package_name} ({}) {installed_text}", self.version);

        let package = if mode.match_binaries() {
            let bins = match_output
                .bins
                .iter()
                .map(|output| {
                    Text::new(format!(
                        "{}{}",
                        crate::output::sectioned::WHITESPACE,
                        output.bold()
                    ))
                })
                .collect_vec();

            Section::new(Children::from(bins))
        } else {
            Section::new(Children::None)
        }
        .with_title(title);

        Some(package)
    }
}

/// Check if the manifest path is installed, and optionally confirm the bucket
///
/// # Panics
/// - The file was not valid UTF-8
pub fn is_installed(manifest_name: impl AsRef<Path>, bucket: Option<impl AsRef<str>>) -> bool {
    let install_path = Scoop::apps_path()
        .join(manifest_name)
        .join("current/install.json");

    match InstallManifest::from_path(install_path) {
        Ok(manifest) => {
            if let Some(bucket) = bucket {
                manifest.get_source() == bucket.as_ref()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
