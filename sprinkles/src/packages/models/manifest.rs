//! Package manifest
// TODO: Add documentation
#![allow(missing_docs)]

// Thanks to quicktype.io for saving me a lot of time.
// The names are a bit weird at times but I'll work on that in future.

mod ser_de;

use std::{collections::HashMap, fmt::Display, ops::Index};

use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::{Architecture, scripts::PowershellScript, version::Version};

#[allow(clippy::unsafe_derive_deserialize)]
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
/// The manifest for a package
pub struct Manifest {
    /// This must be manually set
    #[serde(skip)]
    bucket: Option<String>,
    /// This must be manually set
    #[serde(skip)]
    name: Option<String>,
    /// A comment.
    #[serde(rename = "##")]
    pub empty: Option<StringArray>,
    #[serde(rename = "$schema")]
    /// The schema of the manifest
    pub schema: Option<String>,
    #[deprecated(note = "Use ## instead")]
    #[serde(rename = "_comment")]
    /// A comment.
    pub comment: Option<StringArray>,
    /// The architecture of the package
    pub architecture: Option<ManifestArchitecture>,
    /// The autoupdate configuration
    pub autoupdate: Option<Autoupdate>,
    /// Undocumented: Found at <https://github.com/se35710/scoop-java/search?l=JSON&q=cookie>
    pub cookie: Option<HashMap<String, Option<serde_json::Value>>>,
    /// The dependencies of the package
    pub depends: Option<SingleOrArray<crate::packages::reference::manifest::Reference>>,
    /// The description of the package
    pub description: Option<String>,
    /// Extract to dir or dirs
    pub extract_to: Option<StringArray>,
    /// The homepage of the package
    pub homepage: Option<String>,
    /// True if the installer `InnoSetup` based. Found in
    /// <https://github.com/ScoopInstaller/Main/search?l=JSON&q=innosetup>
    pub innosetup: Option<bool>,
    /// The license of the package
    pub license: Option<PackageLicense>,
    // Deprecated
    /// The manifest notes
    pub notes: Option<StringArray>,
    /// Directories to persist when updating
    pub persist: Option<NestedArray<String>>,
    /// The `PowerShell` module of the package
    pub psmodule: Option<Psmodule>,
    /// The suggested dependencies of the package
    pub suggest: Option<Suggest>,
    /// The version of the package
    pub version: Version,
    /// The environment variables to add to PATH
    pub env_add_path: Option<StringArray>,
    /// The environment variables to set
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    #[serde(flatten)]
    /// The install configuration
    pub install_config: InstallConfig,
}

impl Manifest {
    #[must_use]
    /// Get the name of the manifest
    ///
    /// # Safety
    /// This field is manually set, and by default is uninitialized. This may cause undefined behavior.
    ///
    /// Use [`Manifest::name_opt`] or,
    /// to ensure that this function returns properly, use the [`CreateManifest`] trait to set the name,
    /// or create the manifest, rather than other methods that might fail to set the name.
    pub unsafe fn name(&self) -> &str {
        unsafe { self.name.as_ref().unwrap_unchecked() }
    }

    #[must_use]
    /// Get the name of the manifest, or [`None`] if it is not set
    pub fn name_opt(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the name of the manifest
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = Some(name.into());
    }

    #[must_use]
    /// Get the bucket the manifest is from
    ///
    /// # Safety
    /// This field is manually set, and by default is uninitialized. This may cause undefined behavior.
    ///
    /// Use [`Manifest::bucket_opt`] or,
    /// to ensure that this function returns properly, use the [`CreateManifest`] trait to set the bucket,
    /// or create the manifest, rather than other methods that might fail to set the bucket.
    pub unsafe fn bucket(&self) -> &str {
        unsafe { self.bucket.as_ref().unwrap_unchecked() }
    }

    #[must_use]
    /// Get the bucket the manifest is from, or [`None`] if it is not set
    pub fn bucket_opt(&self) -> Option<&str> {
        self.bucket.as_deref()
    }

    /// Set the bucket the manifest is from
    pub fn set_bucket(&mut self, bucket: impl Into<String>) {
        self.bucket = Some(bucket.into());
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Manifest architecture specific configuration
pub struct ManifestArchitecture {
    #[serde(rename = "32bit")]
    /// The 32-bit configuration
    pub x86: Option<InstallConfig>,
    #[serde(rename = "64bit")]
    /// The 64-bit configuration
    pub x64: Option<InstallConfig>,
    /// The ARM64 configuration
    pub arm64: Option<InstallConfig>,
}

impl std::ops::Index<Architecture> for ManifestArchitecture {
    type Output = Option<InstallConfig>;

    fn index(&self, index: Architecture) -> &Self::Output {
        match index {
            Architecture::Arm64 => &self.arm64,
            Architecture::X64 => &self.x64,
            Architecture::X86 => &self.x86,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// The install configuration
pub struct InstallConfig {
    /// The package binaries
    pub bin: Option<NestedArray<String>>,
    /// The checkver configuration
    pub checkver: Option<Checkver>,
    /// The directories to extract to
    pub extract_dir: Option<StringArray>,
    #[cfg(feature = "manifest-hashes")]
    #[serde(default, deserialize_with = "ser_de::deserialize_hash")]
    /// The hash(es) of the package
    pub hash: Option<SingleOrArray<crate::hash::Hash>>,
    /// The installer configuration
    pub installer: Option<Installer>,
    #[deprecated]
    pub msi: Option<StringArray>,
    pub post_install: Option<PowershellScript>,
    pub post_uninstall: Option<PowershellScript>,
    pub pre_install: Option<PowershellScript>,
    pub pre_uninstall: Option<PowershellScript>,
    pub shortcuts: Option<NestedArray<String>>,
    pub uninstaller: Option<Installer>,
    #[serde(rename = "url")]
    pub urls: Option<StringArray>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckverClass {
    pub github: Option<String>,
    /// Same as 'jsonpath'
    pub jp: Option<String>,
    pub jsonpath: Option<String>,
    /// Same as 'regex'
    pub re: Option<String>,
    pub regex: Option<String>,
    /// Allows rearrange the regexp matches
    pub replace: Option<String>,
    /// Reverse the order of regex matches
    pub reverse: Option<bool>,
    /// Custom `PowerShell` script to retrieve application version using more complex approach.
    pub script: Option<StringArray>,
    pub sourceforge: Option<SourceforgeUnion>,
    pub url: Option<String>,
    pub useragent: Option<String>,
    pub xpath: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceforgeClass {
    pub path: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum InstallerRunner {
    File(String),
    Script(PowershellScript),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Installer {
    /// Undocumented: only used in scoop-extras/oraclejdk* and scoop-extras/appengine-go
    #[serde(rename = "_comment")]
    pub comment: Option<String>,
    pub args: Option<StringArray>,
    pub file: Option<String>,
    pub keep: Option<bool>,
    pub script: Option<PowershellScript>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Autoupdate {
    pub architecture: Option<AutoupdateArchitecture>,
    pub license: Option<AutoupdateLicense>,
    pub notes: Option<StringArray>,
    pub persist: Option<NestedArray<String>>,
    pub psmodule: Option<AutoupdatePsmodule>,
    #[serde(flatten)]
    pub default_config: AutoupdateConfig,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdateArchitecture {
    #[serde(rename = "32bit")]
    pub x86: Option<AutoupdateConfig>,
    #[serde(rename = "64bit")]
    pub x64: Option<AutoupdateConfig>,
    pub arm64: Option<AutoupdateConfig>,
}

impl AutoupdateArchitecture {
    #[must_use]
    pub fn get(&self, arch: Architecture) -> Option<&AutoupdateConfig> {
        match arch {
            Architecture::Arm64 => self.arm64.as_ref(),
            Architecture::X64 => self.x64.as_ref(),
            Architecture::X86 => self.x86.as_ref(),
        }
    }

    #[must_use]
    pub fn get_mut(&mut self, arch: Architecture) -> Option<&mut AutoupdateConfig> {
        match arch {
            Architecture::Arm64 => self.arm64.as_mut(),
            Architecture::X64 => self.x64.as_mut(),
            Architecture::X86 => self.x86.as_mut(),
        }
    }

    #[must_use]
    /// Get the best architecture config
    ///
    /// [`Architecture::Arm64`] will always return the [`AutoupdateConfig`] for [`Architecture::Arm64`]
    ///
    /// [`Architecture::X64`] will return the [`AutoupdateConfig`] for [`Architecture::X64`] if it exists, otherwise it will return the [`AutoupdateConfig`] for [`Architecture::X86`]
    ///
    /// [`Architecture::X86`] will always return the [`AutoupdateConfig`] for [`Architecture::X86`]
    pub fn get_best(&self, arch: Architecture) -> Option<&AutoupdateConfig> {
        match arch {
            Architecture::X64 => self
                .get(Architecture::X64)
                .or_else(|| self.get(Architecture::X86)),
            _ => self.get(arch),
        }
    }
}

impl Index<Architecture> for AutoupdateArchitecture {
    type Output = AutoupdateConfig;

    fn index(&self, index: Architecture) -> &Self::Output {
        self.get(index)
            .expect("autoupdate config missing for architecture")
    }
}

// TODO: Merge fields from AutoupdateConfig into and various Architectures

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdateConfig {
    pub bin: Option<NestedArray<String>>,
    pub env_add_path: Option<StringArray>,
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub extract_dir: Option<StringArray>,
    pub hash: Option<HashExtractionOrArrayOfHashExtractions>,
    pub installer: Option<Installer>,
    pub shortcuts: Option<NestedArray<String>>,
    pub url: Option<StringArray>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HashExtraction {
    /// Same as 'regex'
    pub find: Option<String>,
    /// Same as 'jsonpath'
    pub jp: Option<String>,
    pub jsonpath: Option<String>,
    pub mode: Option<HashMode>,
    pub regex: Option<String>,
    #[deprecated(note = "hash type is determined automatically")]
    #[serde(rename = "type")]
    pub hash_extraction_type: Option<Type>,
    pub url: Option<String>,
    pub xpath: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurpleInstaller {
    pub file: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdateInstaller {
    pub file: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct License {
    pub identifier: String,
    pub url: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdatePsmodule {
    pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Psmodule {
    pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Suggest {}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
/// A single element, or an array, or an array of arrays
pub enum NestedArray<T> {
    NestedArray(SingleOrArray<T>),
    AliasArray(Vec<SingleOrArray<T>>),
}

impl<T> NestedArray<T> {
    #[must_use]
    pub fn from_vec(vec: Vec<Vec<T>>) -> Self {
        let output = vec
            .into_iter()
            .map(SingleOrArray::from_vec_or_default)
            .collect();

        Self::AliasArray(output)
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<T>
    where
        // &'a T: ToOwned<Owned = T>,
        T: Clone,
    {
        match self {
            NestedArray::NestedArray(SingleOrArray::Single(v)) => vec![v.to_owned()],
            NestedArray::NestedArray(SingleOrArray::Array(v)) => v.to_owned(),
            NestedArray::AliasArray(v) => {
                v.iter().cloned().flat_map(SingleOrArray::to_vec).collect()
            }
        }
    }
}

impl<T: Display> Display for NestedArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NestedArray::NestedArray(v) => {
                debug!("wtf bro");
                v.fmt(f)
            }
            NestedArray::AliasArray(alias_array) => alias_array
                .iter()
                .map(|alias| match alias {
                    SingleOrArray::Single(v) => v,
                    SingleOrArray::Array(v) => &v[1],
                })
                .format(", ")
                .fmt(f),
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Checkver {
    CheckverClass(Box<CheckverClass>),
    String(String),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
/// A single element, or an array
// TODO: Implement serializing manually so as to enable serializing null if it is an empty array
pub enum SingleOrArray<T> {
    Single(T),
    Array(Vec<T>),
}

impl<T> SingleOrArray<T> {
    pub fn map<O>(self, f: impl Fn(T) -> O) -> SingleOrArray<O> {
        match self {
            Self::Single(s) => SingleOrArray::Single(f(s)),
            Self::Array(s) => SingleOrArray::Array(s.into_iter().map(f).collect()),
        }
    }

    pub fn to_vec(self) -> Vec<T> {
        match self {
            SingleOrArray::Single(t) => vec![t],
            SingleOrArray::Array(array) => array,
        }
    }

    #[must_use]
    // It won't panic because it is guaranteed to have at least one element
    #[allow(clippy::missing_panics_doc)]
    pub fn from_vec(array: Vec<T>) -> Option<Self> {
        if array.is_empty() {
            None
        } else if array.len() == 1 {
            Some(SingleOrArray::Single(array.into_iter().next().unwrap()))
        } else {
            Some(SingleOrArray::Array(array))
        }
    }

    #[must_use]
    // It won't panic because it is guaranteed to have at least one element
    #[allow(clippy::missing_panics_doc)]
    pub fn from_vec_or_default(array: Vec<T>) -> Self {
        if array.len() == 1 {
            SingleOrArray::Single(array.into_iter().next().unwrap())
        } else {
            SingleOrArray::Array(array)
        }
    }

    pub fn to_option(self) -> Option<Self> {
        match self {
            SingleOrArray::Single(_) => Some(self),
            SingleOrArray::Array(array) => {
                if array.is_empty() {
                    None
                } else {
                    Some(SingleOrArray::Array(array))
                }
            }
        }
    }

    /// Returns Some(T) if it is a single value, None otherwise
    pub fn single(self) -> Option<T> {
        match self {
            SingleOrArray::Single(t) => Some(t),
            SingleOrArray::Array(_) => None,
        }
    }

    /// Returns [`Some`] if it is an array, [`None`] otherwise
    pub fn array(self) -> Option<Vec<T>> {
        match self {
            SingleOrArray::Single(_) => None,
            SingleOrArray::Array(array) => Some(array),
        }
    }
}

impl<A> FromIterator<A> for SingleOrArray<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let vec: Vec<A> = iter.into_iter().collect();

        Self::from_vec_or_default(vec)
    }
}

impl<T: Display> Display for SingleOrArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SingleOrArray::Single(v) => v.fmt(f),
            SingleOrArray::Array(v) => v.iter().format(", ").fmt(f),
        }
    }
}

pub type StringArray = SingleOrArray<String>;

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SourceforgeUnion {
    SourceforgeClass(SourceforgeClass),
    String(String),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HashExtractionOrArrayOfHashExtractions {
    Url(String),
    HashExtraction(HashExtraction),
    // HashExtractionArray(Vec<HashExtraction>),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AutoupdateLicense {
    License(License),
    String(String),
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PackageLicense {
    License(License),
    String(String),
    Object(LicenseObject),
}

impl std::fmt::Display for PackageLicense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageLicense::License(license) => {
                write!(f, "{}", license.identifier)?;
                if let Some(url) = &license.url {
                    write!(f, " ({url})")?;
                }

                Ok(())
            }
            PackageLicense::String(license) => write!(f, "{license}"),
            PackageLicense::Object(LicenseObject {
                identifier: Some(identifier),
                ..
            }) => write!(f, "{identifier}",),
            PackageLicense::Object(LicenseObject { url: Some(url), .. }) => write!(f, "{url}",),
            PackageLicense::Object(_) => write!(f, "Unknown"),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseObject {
    identifier: Option<String>,
    url: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
    #[serde(rename = "md5")]
    Md5,
    #[serde(rename = "sha1")]
    Sha1,
    #[serde(rename = "sha256")]
    Sha256,
    #[serde(rename = "sha512")]
    Sha512,
}

#[skip_serializing_none]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HashMode {
    #[serde(rename = "download")]
    Download,
    #[serde(rename = "extract")]
    Extract,
    #[serde(rename = "fosshub")]
    Fosshub,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "metalink")]
    Metalink,
    #[serde(rename = "rdf")]
    Rdf,
    #[serde(rename = "sourceforge")]
    Sourceforge,
    #[serde(rename = "xpath")]
    Xpath,
}

#[cfg(test)]
mod tests {
    use super::Manifest;

    #[test]
    fn test_manifest_serde() {
        const MANIFEST: &str = r#"{"version":"0.10.1","description":"General-purpose programming language designed for robustness, optimality, and maintainability.","homepage":"https://ziglang.org/","license":"MIT","suggest":{"vcredist":"extras/vcredist2022"},"architecture":{"64bit":{"url":"https://ziglang.org/download/0.10.1/zig-windows-x86_64-0.10.1.zip","hash":"5768004e5e274c7969c3892e891596e51c5df2b422d798865471e05049988125","extract_dir":"zig-windows-x86_64-0.10.1"},"arm64":{"url":"https://ziglang.org/download/0.10.1/zig-windows-aarch64-0.10.1.zip","hash":"ece93b0d77b2ab03c40db99ef7ccbc63e0b6bd658af12b97898960f621305428","extract_dir":"zig-windows-aarch64-0.10.1"}},"bin":"zig.exe","checkver":{"url":"https://ziglang.org/download/","regex":">([\\d.]+)</h"},"autoupdate":{"architecture":{"64bit":{"url":"https://ziglang.org/download/$version/zig-windows-x86_64-$version.zip","extract_dir":"zig-windows-x86_64-$version"},"arm64":{"url":"https://ziglang.org/download/$version/zig-windows-aarch64-$version.zip","extract_dir":"zig-windows-aarch64-$version"}},"hash":{"url":"https://ziglang.org/download/","regex":"(?sm)$basename.*?$sha256"}}}"#;

        let deserialized: Manifest = serde_json::from_str(MANIFEST).unwrap();

        dbg!(deserialized);
    }

    #[test]
    fn test_equal_generated_manifests() {
        const SCOOP_GENERATED: &str = r#"{"version":"1.10.1","description":"Stupid Fast Scoop Utilities. Incredibly fast replacements for commonly used Scoop commands, written in Rust.","homepage":"https://github.com/winpax/sfsu","license":"Apache-2.0","architecture":{"64bit":{"url":"https://github.com/winpax/sfsu/releases/download/v1.10.1/sfsu-x86_64.exe#/sfsu.exe","hash":"e2a1c7dd49d547fdfe05fc45f0c9e276cb992bd94af151f0cf7d3e2ecfdc4233"},"32bit":{"url":"https://github.com/winpax/sfsu/releases/download/v1.10.1/sfsu-i686.exe#/sfsu.exe","hash":"b40478dc261fb58caecadd058dc7897a65167ca1f43993908b12dd389790dbd5"},"arm64":{"url":"https://github.com/winpax/sfsu/releases/download/v1.10.1/sfsu-aarch64.exe#/sfsu.exe","hash":"17d813fd810d074fd52bd9da8aabc6e52cf27d78a34d2b4403025d5da4b0e13d"}},"notes":"In order to replace scoop commands use `Invoke-Expression (&sfsu hook)` in your Powershell profile.","bin":"sfsu.exe","checkver":"github","autoupdate":{"architecture":{"64bit":{"url":"https://github.com/winpax/sfsu/releases/download/v$version/sfsu-x86_64.exe#/sfsu.exe"},"32bit":{"url":"https://github.com/winpax/sfsu/releases/download/v$version/sfsu-i686.exe#/sfsu.exe"},"arm64":{"url":"https://github.com/winpax/sfsu/releases/download/v$version/sfsu-aarch64.exe#/sfsu.exe"}},"hash":{"url":"$url.sha256"}}}"#;

        const SFSU_GENERATED: &str = r#"{"architecture":{"32bit":{"hash":"b40478dc261fb58caecadd058dc7897a65167ca1f43993908b12dd389790dbd5","url":"https://github.com/winpax/sfsu/releases/download/v1.10.1/sfsu-i686.exe#/sfsu.exe"},"64bit":{"hash":"e2a1c7dd49d547fdfe05fc45f0c9e276cb992bd94af151f0cf7d3e2ecfdc4233","url":"https://github.com/winpax/sfsu/releases/download/v1.10.1/sfsu-x86_64.exe#/sfsu.exe"},"arm64":{"hash":"17d813fd810d074fd52bd9da8aabc6e52cf27d78a34d2b4403025d5da4b0e13d","url":"https://github.com/winpax/sfsu/releases/download/v1.10.1/sfsu-aarch64.exe#/sfsu.exe"}},"autoupdate":{"architecture":{"32bit":{"url":"https://github.com/winpax/sfsu/releases/download/v$version/sfsu-i686.exe#/sfsu.exe"},"64bit":{"url":"https://github.com/winpax/sfsu/releases/download/v$version/sfsu-x86_64.exe#/sfsu.exe"},"arm64":{"url":"https://github.com/winpax/sfsu/releases/download/v$version/sfsu-aarch64.exe#/sfsu.exe"}},"hash":{"url":"$url.sha256"}},"description":"Stupid Fast Scoop Utilities. Incredibly fast replacements for commonly used Scoop commands, written in Rust.","homepage":"https://github.com/winpax/sfsu","license":"Apache-2.0","notes":"In order to replace scoop commands use `Invoke-Expression (&sfsu hook)` in your Powershell profile.","version":"1.10.1","bin":"sfsu.exe","checkver":"github"}"#;

        let scoop_generated: Manifest = serde_json::from_str(SCOOP_GENERATED).unwrap();
        let sfsu_generated: Manifest = serde_json::from_str(SFSU_GENERATED).unwrap();

        assert_eq!(scoop_generated, sfsu_generated);
    }
}
