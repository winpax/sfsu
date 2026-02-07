//! Scoop package helpers

#![allow(edition_2024_expr_fragment_specifier)]

use std::{path::Path, time::SystemTimeError};

use chrono::{DateTime, FixedOffset};
use gix::{
    object::tree::diff::Action, revision::walk::Sorting, traverse::commit::simple::CommitTimeOrder,
};
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use strum::Display;

use crate::{
    Architecture,
    buckets::{self, Bucket},
    contexts::ScoopContext,
    git::{
        self, Repo,
        errors::{self, GitoxideError},
        parity::Signature,
    },
};

#[cfg(feature = "manifest-hashes")]
use crate::{
    hash::{
        self,
        substitutions::{Substitute, SubstitutionMap},
    },
    packages::manifest::SingleOrArray,
};

pub mod array;
pub mod downloading;
pub mod installer;
pub mod models;
pub mod reference;

pub use models::{install::Manifest as InstallManifest, manifest::Manifest};

use downloading::DownloadUrl;
use models::manifest::{InstallConfig, StringArray};

#[macro_use]
mod macros {
    /// Get a field from a manifest based on the architecture
    macro_rules! arch_config {
        ($field:ident.$arch:expr_2021) => {
            match $arch {
                $crate::Architecture::Arm64 => $field.arm64.as_ref(),
                $crate::Architecture::X64 => $field.x64.as_ref(),
                $crate::Architecture::X86 => $field.x86.as_ref(),
            }
        };

        ($field:ident) => {
            arch_config!($field.$crate::Architecture::ARCH)
        };

        ($field:ident.$arch:expr_2021 => clone) => {
            arch_config!($field.$arch).cloned()
        };

        ($field:ident => clone) => {
            arch_config!($field).cloned()
        };

        // ($field:ident.$arch:expr => $default:expr) => {
        //     arch_config!($field.$arch).unwrap_or($default)
        // };

        // ($field:ident => $default:expr) => {
        //     arch_config!($field.$crate::Architecture::ARCH).unwrap_or($default)
        // };
    }

    /// Get a field from a manifest based on the architecture
    macro_rules! arch_field {
        // ($self:ident.$field:ident) => {
        //     arch_field!($self.$field).clone()
        // };

        // ($arch:expr => ref $self:ident.$field:ident) => {{
        //     if let Some(cfg) = match $arch {
        //         $crate::Architecture::Arm64 => &$self.arm64,
        //         $crate::Architecture::X64 => &$self.x64,
        //         $crate::Architecture::X86 => &$self.x86,
        //     } {
        //         &cfg.$field
        //     } else {
        //         &None
        //     }
        // }};

        // (ref $self:ident.$field:ident) => {
        //     arch_field!($crate::Architecture::ARCH => ref $self.$field)
        // };

        // ($arch:expr => ref mut $self:ident.$field:ident) => {{
        //     match $arch {
        //         $crate::Architecture::Arm64 => $self.arm64.as_mut(),
        //         $crate::Architecture::X64 => $self.x64.as_mut(),
        //         $crate::Architecture::X86 => $self.x86.as_mut(),
        //     }.and_then(|cfg| cfg.$field.as_mut())
        // }};

        // (ref mut $self:ident.$field:ident) => {
        //     arch_field!($crate::Architecture::ARCH => ref mut $self.$field)
        // };

        ($self:ident.$field:ident as cloned) => {
            arch_field!($crate::Architecture::ARCH => $self.$field as ref).cloned()
        };

        ($arch:expr_2021 => $self:ident.$field:ident as cloned) => {
            arch_field!($arch => $self.$field as ref).cloned()
        };

        ($self:ident.$field:ident as ref) => {
            arch_field!($crate::Architecture::ARCH => $self.$field as ref)
        };

        ($arch:expr_2021 => $self:ident.$field:ident as ref) => {{
            match $arch {
                $crate::Architecture::Arm64 => $self.arm64.as_ref(),
                $crate::Architecture::X64 => $self.x64.as_ref(),
                $crate::Architecture::X86 => $self.x86.as_ref(),
            }.and_then(|cfg| cfg.$field.as_ref())
        }};

        ($self:ident.$field:ident as mut) => {
            arch_field!($crate::Architecture::ARCH => $self.$field as mut)
        };

        ($arch:expr_2021 => $self:ident.$field:ident as mut) => {{
            match $arch {
                $crate::Architecture::Arm64 => $self.arm64.as_mut(),
                $crate::Architecture::X64 => $self.x64.as_mut(),
                $crate::Architecture::X86 => $self.x86.as_mut(),
            }.and_then(|cfg| cfg.$field.as_mut())
        }};
    }
}

use self::models::manifest::{
    self, AutoupdateArchitecture, AutoupdateConfig, HashExtraction,
    HashExtractionOrArrayOfHashExtractions, ManifestArchitecture, NestedArray,
};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Package error
pub enum Error {
    #[error("Invalid utf8 found. This is not supported by sfsu")]
    NonUtf8,
    #[error("Could not find parent tree")]
    MissingParentTree,
    #[error("Missing or invalid file name. The path terminated in '..' or wasn't valid utf8")]
    MissingFileName,
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("Could not parse manifest \"{0}\". Failed with error: {1}")]
    ParsingManifest(String, serde_json::Error),
    #[error("Interacting with buckets: {0}")]
    BucketError(#[from] buckets::Error),
    #[error("Interacting with git2: {0}")]
    RepoError(#[from] git::Error),
    #[error("git2 internal error: {0}")]
    Git2Error(#[from] git2::Error),
    #[error("System Time: {0}")]
    TimeError(#[from] SystemTimeError),
    #[error("Could not find executable in path: {0}")]
    MissingInPath(#[from] which::Error),
    #[error("Gitoxide error: {0}")]
    Gitoxide(#[from] Box<errors::GitoxideError>),
    #[error("Git delta did not have a path")]
    DeltaNoPath,
    #[error("Cannot find git commit where package was updated")]
    NoUpdatedCommit,
    #[error(
        "Invalid time. (time went backwards or way way way too far forwards (hello future! whats it like?))"
    )]
    InvalidTime,
    #[error("Invalid timezone provided. (where are you?)")]
    InvalidTimeZone,
    #[error("Git provided no output")]
    MissingGitOutput,
    #[error("Missing local manifest for package")]
    MissingLocalManifest,
    #[cfg(feature = "manifest-hashes")]
    #[error("Could not get hash for app: {0}")]
    HashError(#[from] hash::Error),
    #[error("Manifest does not have `autoupdate` field")]
    MissingAutoUpdate,
    #[error("Manifest architecture section does not have `autoupdate` field")]
    MissingArchAutoUpdate,
    #[error("Commit did not have a parent")]
    MissingParent,
}

impl From<errors::GitoxideError> for Error {
    fn from(value: errors::GitoxideError) -> Self {
        Self::Gitoxide(Box::new(value))
    }
}

/// The result type for package operations
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Default, Copy, Clone, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
/// The search mode
pub enum SearchMode {
    #[default]
    /// Only search the name
    Name,
    /// Only search the binaries
    Binary,
    /// Search both the name and binaries
    Both,
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for SearchMode {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Name, Self::Binary, Self::Both]
    }
    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Name => {
                Some(clap::builder::PossibleValue::new("name").help("Only search the name"))
            }
            Self::Binary => Some({
                clap::builder::PossibleValue::new("binary").help("Only search the binaries")
            }),
            Self::Both => Some({
                clap::builder::PossibleValue::new("both").help("Search both the name and binaries")
            }),
        }
    }
}

impl SearchMode {
    #[must_use]
    /// Check if the search mode matches names
    pub fn match_names(self) -> bool {
        matches!(self, SearchMode::Name | SearchMode::Both)
    }

    #[must_use]
    /// Check if the search mode only matches names
    pub fn only_match_names(self) -> bool {
        self == SearchMode::Name
    }

    #[must_use]
    /// Check if the search mode matches binaries
    pub fn match_binaries(self) -> bool {
        matches!(self, SearchMode::Binary | SearchMode::Both)
    }

    #[must_use]
    /// Check if the search mode only matches binaries
    pub fn only_match_binaries(self) -> bool {
        self == SearchMode::Binary
    }

    #[must_use]
    /// Check if the search mode matches both names and binaries
    ///
    /// Checks name first to avoid unnecessary binary checks
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

/// Localised functions for creating manifests
pub trait CreateManifest
where
    Self: for<'a> Deserialize<'a>,
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
            .map(|s| s.with_name(path).with_bucket(path))
            .map_err(|e| Error::ParsingManifest(path.display().to_string(), e))
    }

    /// # Errors
    /// - The contents are not a valid manifest
    fn from_str(contents: impl AsRef<str>) -> serde_json::Result<Self> {
        let trimmed = contents.as_ref().trim_start_matches('\u{feff}');

        serde_json::from_str(trimmed)
    }

    /// Set the name of the manifest. Not meant to be used directly.
    #[must_use]
    fn with_name(self, path: impl AsRef<Path>) -> Self;

    /// Set the bucket of the manifest. Not meant to be used directly.
    #[must_use]
    fn with_bucket(self, path: impl AsRef<Path>) -> Self;
}

impl CreateManifest for Manifest {
    fn with_name(mut self, path: impl AsRef<Path>) -> Self {
        let ext_stripped = path.as_ref().with_extension("");

        let name = ext_stripped.file_name().map(|f| f.to_string_lossy());

        if let Some(name) = name {
            if name == "manifest" || name == "install" {
                let mut path_buf = path.as_ref().to_path_buf();

                if path_buf.pop()
                    && path_buf.pop()
                    && let Some(name) = path_buf.file_name()
                {
                    self.set_name(name.to_string_lossy());
                }
            }

            self.set_name(name);
        }

        self
    }

    fn with_bucket(mut self, path: impl AsRef<Path>) -> Self {
        if let Some(bucket) = path
            .as_ref()
            .parent()
            .and_then(|p| p.parent())
            .and_then(|bucket| bucket.file_name().map(|f| f.to_string_lossy().to_string()))
        {
            self.set_bucket(bucket);
        }

        self
    }
}

impl CreateManifest for InstallManifest {
    fn with_name(mut self, path: impl AsRef<Path>) -> Self {
        if let Some(name) = path
            .as_ref()
            .with_extension("")
            .file_name()
            .map(|f| f.to_string_lossy())
        {
            self.set_name(name);
        }

        self
    }

    fn with_bucket(self, _path: impl AsRef<Path>) -> Self {
        self
    }
}

impl InstallManifest {
    /// List all install manifests
    ///
    /// # Errors
    /// - Invalid install manifest
    /// - Reading directories fails
    pub fn list_all(ctx: &impl ScoopContext) -> Result<Vec<Self>> {
        let installed_apps = ctx.installed_apps()?;
        {
            cfg_if::cfg_if! {
                if #[cfg(feature = "rayon")] {
                    installed_apps.par_iter()
                } else {
                    installed_apps.iter()
                }
            }
        }
        .map(|path| Self::from_path(path.join("current/install.json")))
        .collect::<Result<Vec<_>>>()
    }

    /// List all install manifests, ignoring errors
    ///
    /// # Errors
    /// - Reading directories fails
    pub fn list_all_unchecked(ctx: &impl ScoopContext) -> Result<Vec<Self>> {
        let installed_apps = ctx.installed_apps()?;

        Ok({
            cfg_if::cfg_if! {
                if #[cfg(feature = "rayon")] {
                    installed_apps.par_iter()
                } else {
                    installed_apps.iter()
                }
            }
        }
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
    /// Get the install config for a given architecture
    pub fn install_config(&self, arch: Architecture) -> InstallConfig {
        self.architecture
            .as_ref()
            .merge_default(self.install_config.clone(), arch)
    }

    #[must_use]
    /// Get the autoupdate config for the default architecture
    pub fn autoupdate_config(&self, arch: Architecture) -> Option<AutoupdateConfig> {
        let autoupdate = self.autoupdate.as_ref()?;

        Some(
            autoupdate
                .architecture
                .clone()
                .merge_default(autoupdate.default_config.clone(), arch),
        )
    }

    #[must_use]
    /// Get the download urls for a given architecture
    pub fn download_urls(&self, arch: Architecture) -> Option<Vec<DownloadUrl>> {
        let urls = self.install_config(arch).urls?;

        Some(
            urls.to_vec()
                .into_iter()
                .map(DownloadUrl::from_string)
                .collect(),
        )
    }

    #[must_use]
    /// Apply a bucket to a manifest
    pub fn with_bucket(mut self, bucket: &Bucket) -> Self {
        self.set_bucket(bucket.name());

        self
    }

    #[must_use]
    /// List the dependencies of a given manifest, in the order that they will be installed
    ///
    /// Note that this does not include the package itself as a dependency
    pub fn depends(&self) -> Vec<reference::manifest::Reference> {
        self.depends
            .clone()
            .map(manifest::SingleOrArray::to_vec)
            .unwrap_or_default()
    }

    /// Gets the manifest from a bucket and manifest name
    ///
    /// # Errors
    /// - If the manifest doesn't exist or bucket is invalid
    pub fn from_reference(
        ctx: &impl ScoopContext,
        (bucket, name): (String, String),
    ) -> Result<Self> {
        Bucket::from_name(ctx, bucket)?.get_manifest(name)
    }

    #[must_use]
    /// Check if the manifest binaries matche the given regex
    pub fn binary_matches(&self, regex: &Regex, arch: Architecture) -> Option<Vec<String>> {
        match self
            .architecture
            .as_ref()
            .merge_default(self.install_config.clone(), arch)
            .bin
        {
            Some(NestedArray::NestedArray(StringArray::Single(ref binary))) => {
                if regex.is_match(binary) {
                    Some(vec![binary.clone()])
                } else {
                    None
                }
            }
            Some(NestedArray::NestedArray(StringArray::Array(ref binaries))) => {
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
    pub fn list_installed(ctx: &impl ScoopContext) -> Result<Vec<Result<Self>>> {
        let installed_apps = ctx.installed_apps()?;

        Ok({
            cfg_if::cfg_if! {
                if #[cfg(feature = "rayon")] {
                    installed_apps.par_iter()
                } else {
                    installed_apps.iter()
                }
            }
        }
        .map(|path| {
            Self::from_path(path.join("current/manifest.json")).and_then(|mut manifest| {
                manifest.set_name(
                    path.file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .ok_or(Error::MissingFileName)?,
                );

                Ok(manifest)
            })
        })
        .collect::<Vec<_>>())
    }

    #[must_use]
    /// Check if the manifest is installed
    pub fn is_installed(&self, ctx: &impl ScoopContext, bucket: Option<&str>) -> bool {
        is_installed(ctx, unsafe { self.name() }, bucket)
    }

    fn update_field<T>(
        arch_field: Option<&mut T>,
        default_field: &mut Option<T>,
        value: Option<T>,
    ) {
        if let Some(arch_field) = arch_field
            && let Some(value) = value
        {
            *arch_field = value;
        } else {
            *default_field = value;
        }
        // if let Some(arch_field) = arch_field
        //     && let Some(value) = value
        // {
        //     *arch_field = value;
        // } else {
        //     *default_field = value;
        // }
    }

    #[cfg(feature = "manifest-hashes")]
    fn get_new_urls(&self, autoupdate: &AutoupdateConfig) -> Option<SingleOrArray<String>> {
        use crate::hash::substitutions::Substitute;

        if let Some(autoupdate_urls) = &autoupdate.url {
            debug!("Autoupdate Urls: {autoupdate_urls}");

            let mut submap = SubstitutionMap::new();
            submap.append_version(&self.version);

            let new_urls = autoupdate_urls
                .clone()
                .to_vec()
                .into_iter()
                .map(|url| url.into_substituted(&submap, false));

            Some(new_urls.collect())
        } else {
            None
        }
    }

    #[cfg(feature = "manifest-hashes")]
    /// Set the manifest version and get the hash for the manifest
    ///
    /// # Errors
    /// - Missing autoupdate field
    /// - Hash error
    pub async fn set_version(
        &mut self,
        ctx: &impl ScoopContext,
        version: String,
    ) -> Result<(), Error> {
        use quork::traits::list::ListVariants;

        use crate::hash::Hash;

        let version = version.into();

        if self.version == version {
            // We don't need to do anything
            return Ok(());
        }

        self.version = version;

        let autoupdate = self.autoupdate.as_ref().ok_or(Error::MissingAutoUpdate)?;

        // TODO: This sets the same hash and url for all architectures
        for arch in crate::Architecture::VARIANTS {
            let arch_autoupdate = autoupdate
                .architecture
                .merge_default(autoupdate.default_config.clone(), arch);

            let arch_url = self.get_new_urls(&arch_autoupdate);

            if let Some(arch_config) = &mut self.architecture {
                Self::update_field(
                    arch_field!(arch => arch_config.urls as mut),
                    &mut self.install_config.urls,
                    arch_url,
                );
            } else {
                self.install_config.urls = self.get_new_urls(&autoupdate.default_config);
            }
        }

        macro_rules! update_field {
            ($field:ident) => {{
                let mut submap = SubstitutionMap::new();
                submap.append_version(&self.version);

                for arch in crate::Architecture::VARIANTS {
                    if let Some(config) = &mut self.architecture {
                        let default = arch_field!(arch => config.$field as cloned).map(|s| s.into_substituted(&submap, false));

                        Self::update_field(
                            arch_field!(arch => config.$field as mut),
                            &mut self.install_config.$field,
                            default,
                        );
                    } else {
                        self.install_config.$field = autoupdate.default_config.$field.clone().map(|s| s.into_substituted(&submap, false));
                    }
                }
            }}
        }

        update_field!(bin);
        update_field!(extract_dir);
        update_field!(installer);
        update_field!(shortcuts);

        for arch in crate::Architecture::VARIANTS {
            let Ok(hashes) = Hash::get_for_app(ctx, self, arch).await else {
                continue;
            };

            if let Some(arch_config) = &mut self.architecture {
                // TODO: This sets the same hash and url for all architectures
                Self::update_field(
                    arch_field!(arch => arch_config.hash as mut),
                    &mut self.install_config.hash,
                    SingleOrArray::from_vec(hashes),
                );
            } else {
                self.install_config.hash = SingleOrArray::from_vec(hashes);
            }
        }

        // TODO: Handle other autoupdate fields
        // TODO: Autoupdate fields in all architectures
        // todo!("Handle urls and other autoupdate fields");

        // TODO: Figure out hash extraction
        // autoupdate_arch.hash

        // todo!()

        let workspace_manifest_path = ctx
            .workspace_path()
            .join(format!("{}.json", unsafe { self.name() }));
        serde_json::to_writer_pretty(std::fs::File::create(workspace_manifest_path)?, &self)
            .map_err(|e| {
                error!("Failed to write workspace manifest: {e}");
                Error::ParsingManifest(unsafe { self.name() }.to_string(), e)
            })?;

        Ok(())
    }

    #[must_use]
    /// Check if the commit's message matches the name of the manifest
    pub fn commit_message_matches(&self, commit: &gix::Commit<'_>) -> bool {
        if let Ok(message) = commit.message() {
            message
                .summary()
                .to_string()
                .starts_with(unsafe { self.name() })
        } else {
            false
        }
    }

    /// Check if the commit's changed files matches the name of the manifest
    ///
    /// # Errors
    /// - Git2 errors
    pub fn commit_diff_matches(&self, commit: &gix::Commit<'_>) -> Result<bool> {
        let tree = commit.tree().map_err(GitoxideError::from)?;
        let parent_tree = commit
            .parent_ids()
            .find_map(|parent| {
                match parent.try_object() {
                    Ok(Some(object)) => Some(object),
                    _ => None,
                }
                .and_then(|object| object.peel_to_tree().ok())
            })
            .ok_or(Error::MissingParentTree)?;

        let mut changed = false;

        tree.changes()
            .map_err(GitoxideError::from)?
            .for_each_to_obtain_tree(&parent_tree, |change| {
                // Check if the changed file's location starts with the manifest name
                if change
                    .location()
                    .to_string()
                    .starts_with(unsafe { self.name() })
                {
                    changed = true;
                    return Ok::<_, GitoxideError>(Action::Break(()));
                }

                Ok(Action::Continue(()))
            })
            .map_err(GitoxideError::from)?;

        // Given that the diffoptions ensure that we only match the specific manifest
        // we are safe to return as soon as we find a commit thats changed anything
        Ok(changed)
    }

    /// Get the time and author of the commit where this manifest was last changed
    ///
    /// # Errors
    /// - Invalid bucket
    /// - Invalid repo bucket
    /// - Internal git2 errors
    pub fn last_updated_info(
        &self,
        ctx: &impl ScoopContext,
    ) -> Result<(Option<DateTime<FixedOffset>>, Option<Signature>)> {
        let bucket = Bucket::from_name(ctx, unsafe { self.bucket() })?;

        let repo = Repo::from_bucket(&bucket)?;
        let gitoxide = repo.gitoxide();
        let latest_commit = gitoxide.head_commit().map_err(git::Error::from)?;

        let revwalk = gitoxide
            .rev_walk([latest_commit.id])
            .sorting(Sorting::ByCommitTime(CommitTimeOrder::NewestFirst));

        let updated_commit = revwalk
            .all()
            .map_err(git::Error::from)?
            // .skip(1)
            .find_map(|info| {
                let find_commit = || {
                    // TODO: Add tests using personal bucket to ensure that different methods return the same info
                    let info = info.map_err(git::Error::from)?;
                    let commit = info.object().map_err(git::Error::from)?;

                    // Check for commit message matches first
                    if self.commit_message_matches(&commit) {
                        return Ok(commit);
                    }

                    let mut matches = false;

                    let other = info.parent_ids().next().ok_or(Error::MissingParent)?;
                    let other = other.object().map_err(git::Error::from)?;
                    let other_tree = other.peel_to_tree().map_err(git::Error::from)?;
                    commit
                        .tree()
                        .map_err(git::Error::from)?
                        .changes()
                        .map_err(git::Error::from)?
                        .for_each_to_obtain_tree(&other_tree, |change| {
                            debug!(
                                "Checking change: {:?} at location: {}",
                                change,
                                change.location()
                            );
                            if change
                                .location()
                                .to_string()
                                .starts_with(unsafe { self.name() })
                            {
                                matches = true;
                                Ok::<_, Error>(Action::Break(()))
                            } else {
                                Ok(Action::Continue(()))
                            }
                        })
                        .map_err(git::Error::from)?;

                    if matches {
                        return Ok(commit);
                    }

                    Err(Error::NoUpdatedCommit)
                };

                let result = find_commit();

                match result {
                    Ok(commit) => Some(Ok(commit)),
                    Err(Error::NoUpdatedCommit) => None,
                    Err(e) => Some(Err(e)),
                }
            })
            .ok_or(Error::NoUpdatedCommit)??;

        let date_time = git::parity::Time::from(
            updated_commit
                .time()
                .map_err(git::errors::GitoxideError::from)
                .map_err(Box::new)?,
        )
        .to_datetime()
        .ok_or(Error::InvalidTime)?;

        let author_wrapped =
            Signature::try_from(updated_commit.author().map_err(git::Error::from)?)
                .map_err(git::Error::from)?;

        Ok((Some(date_time), Some(author_wrapped)))
    }

    /// Get [`InstallManifest`] for [`Manifest`]
    ///
    /// # Errors
    /// - Missing or invalid [`InstallManifest`]
    pub fn install_manifest(&self, ctx: &impl ScoopContext) -> Result<InstallManifest> {
        let apps_path = ctx.apps_path();
        let install_path = apps_path
            .join(unsafe { self.name() })
            .join("current")
            .join("install.json");

        debug!("Getting install manifest for {}", install_path.display());

        InstallManifest::from_path(install_path)
    }
}

/// Check if the manifest path is installed, and optionally confirm the bucket
///
/// # Panics
/// - The file was not valid UTF-8
pub fn is_installed(
    ctx: &impl ScoopContext,
    manifest_name: impl AsRef<Path>,
    bucket: Option<impl AsRef<str>>,
) -> bool {
    let install_path = ctx
        .apps_path()
        .join(manifest_name)
        .join("current/install.json");

    match bucket {
        Some(bucket) => {
            matches!(InstallManifest::from_path(install_path), Ok(manifest) if manifest.get_source() == bucket.as_ref())
        }
        _ => install_path.exists(),
    }
}

/// Merge defaults for a given architecture and the provided field
pub trait MergeDefaults {
    /// Output & Input type
    type Default;

    /// Merge the architecture specific autoupdate config with the arch agnostic one
    fn merge_default(&self, default: Self::Default, arch: Architecture) -> Self::Default;
}

impl MergeDefaults for Option<AutoupdateArchitecture> {
    type Default = AutoupdateConfig;

    /// Merge the architecture specific autoupdate config with the arch agnostic one
    fn merge_default(&self, default: Self::Default, arch: Architecture) -> Self::Default {
        let Some(config) = self
            .as_ref()
            .and_then(|config| arch_config!(config.arch => clone))
        else {
            return default;
        };

        AutoupdateConfig {
            bin: config.bin.or(default.bin),
            env_add_path: config.env_add_path.or(default.env_add_path),
            env_set: config.env_set.or(default.env_set),
            extract_dir: config.extract_dir.or(default.extract_dir),
            hash: config.hash.or(default.hash),
            installer: config.installer.or(default.installer),
            shortcuts: config.shortcuts.or(default.shortcuts),
            url: config.url.or(default.url),
        }
    }
}

impl MergeDefaults for Option<&ManifestArchitecture> {
    type Default = InstallConfig;

    /// Merge the architecture specific autoupdate config with the arch agnostic one
    fn merge_default(&self, default: Self::Default, arch: Architecture) -> Self::Default {
        let Some(config) = self
            .as_ref()
            .and_then(|config| arch_config!(config.arch => clone))
        else {
            return default;
        };

        #[allow(deprecated)]
        InstallConfig {
            bin: config.bin.or(default.bin),
            checkver: config.checkver.or(default.checkver),
            extract_dir: config.extract_dir.or(default.extract_dir),
            #[cfg(feature = "manifest-hashes")]
            hash: config.hash.or(default.hash),
            installer: config.installer.or(default.installer),
            msi: config.msi.or(default.msi),
            post_install: config.post_install.or(default.post_install),
            post_uninstall: config.post_uninstall.or(default.post_uninstall),
            pre_install: config.pre_install.or(default.pre_install),
            pre_uninstall: config.pre_uninstall.or(default.pre_uninstall),
            shortcuts: config.shortcuts.or(default.shortcuts),
            uninstaller: config.uninstaller.or(default.uninstaller),
            urls: config.urls.or(default.urls),
        }
    }
}

impl MergeDefaults for Option<ManifestArchitecture> {
    type Default = InstallConfig;

    /// Merge the architecture specific autoupdate config with the arch agnostic one
    fn merge_default(&self, default: Self::Default, arch: Architecture) -> Self::Default {
        self.as_ref().merge_default(default, arch)
    }
}

impl HashExtractionOrArrayOfHashExtractions {
    #[must_use]
    /// Get the hash extraction as a single hash extraction object
    pub fn as_object(&self) -> Option<&HashExtraction> {
        match self {
            Self::Url(_) => None,
            Self::HashExtraction(hash) => Some(hash),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{Architecture, buckets::Bucket, contexts::User};

    use quork::prelude::ListVariants;
    use rayon::prelude::*;

    #[test]
    fn test_parse_all_manifests() -> Result<(), Box<dyn Error>> {
        const UNSUPPORTED_PACKAGES: &[&str] = &["unityhub"];

        let buckets = Bucket::list_all(&User::new().unwrap())?;

        let manifests = buckets
            .into_par_iter()
            .flat_map(|bucket| bucket.list_packages())
            .flatten()
            .filter(|manifest| !UNSUPPORTED_PACKAGES.contains(&unsafe { manifest.name() }))
            .filter(|manifest| manifest.autoupdate_config(Architecture::ARCH).is_some())
            .collect::<Vec<_>>();

        manifests.par_iter().for_each(|manifest| {
            assert!(!unsafe { manifest.name() }.is_empty());
            assert!(!unsafe { manifest.bucket() }.is_empty());

            // TODO: This is a hack ignore it please ill figure something out for
            let mut found_literally_any_url = false;

            if let Some(autoupdate_architecture) = manifest
                .autoupdate
                .as_ref()
                .and_then(|autoupdate| autoupdate.architecture.as_ref())
            {
                for arch in crate::Architecture::VARIANTS {
                    if let Some(autoupdate_config) = autoupdate_architecture.get(arch)
                        && autoupdate_config.url.is_some()
                    {
                        found_literally_any_url = true;
                        break;
                    }
                }
            }

            #[allow(if_let_rescope)]
            if let Some(autoupdate_config) = &manifest.autoupdate_config(Architecture::ARCH) {
                if autoupdate_config.url.is_some() {
                    found_literally_any_url = true;
                }

                // This has to be inside this if statement
                // because otherwise the check will happen for manifests without autoupdate configs
                assert!(
                    found_literally_any_url,
                    "URL is missing in package: {}",
                    unsafe { manifest.name() }
                );
            }
        });

        Ok(())
    }
}
