//! Package manifest
// TODO: Add documentation
#![allow(missing_docs)]

// Thanks to quicktype.io for saving me a lot of time.
// The names are a bit weird at times but I'll work on that in future.

use std::{collections::HashMap, fmt::Display};

use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

use crate::Architecture;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
/// The manifest for a package
pub struct Manifest {
    /// This must be manually set
    #[serde(skip)]
    pub bucket: String,
    /// This must be manually set
    #[serde(skip)]
    pub name: String,
    /// A comment.
    #[serde(rename = "##")]
    pub empty: Option<StringOrArrayOfStrings>,
    #[serde(rename = "$schema")]
    /// The schema of the manifest
    pub schema: Option<String>,
    #[deprecated(since = "1.10.0", note = "Use ## instead")]
    #[serde(rename = "_comment")]
    /// A comment.
    pub comment: Option<StringOrArrayOfStrings>,
    /// The architecture of the package
    pub architecture: Option<ManifestArchitecture>,
    /// The autoupdate configuration
    pub autoupdate: Option<Autoupdate>,
    /// Undocumented: Found at https://github.com/se35710/scoop-java/search?l=JSON&q=cookie
    pub cookie: Option<HashMap<String, Option<serde_json::Value>>>,
    /// The dependencies of the package
    pub depends: Option<TOrArrayOfTs<super::reference::Package>>,
    /// The description of the package
    pub description: Option<String>,
    /// Extract to dir or dirs
    pub extract_to: Option<StringOrArrayOfStrings>,
    /// The homepage of the package
    pub homepage: Option<String>,
    /// True if the installer InnoSetup based. Found in
    /// https://github.com/ScoopInstaller/Main/search?l=JSON&q=innosetup
    pub innosetup: Option<bool>,
    /// The license of the package
    pub license: Option<PackageLicense>,
    // Deprecated
    /// The manifest notes
    pub notes: Option<StringOrArrayOfStrings>,
    /// Directories to persist when updating
    pub persist: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    /// The PowerShell module of the package
    pub psmodule: Option<Psmodule>,
    /// The suggested dependencies of the package
    pub suggest: Option<Suggest>,
    /// The version of the package
    pub version: String,
    /// The package binaries
    pub bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    /// The checkver configuration
    pub checkver: Option<Checkver>,
    /// The environment variables to add to PATH
    pub env_add_path: Option<StringOrArrayOfStrings>,
    /// The environment variables to set
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    /// The directories to extract to
    pub extract_dir: Option<StringOrArrayOfStrings>,
    /// The hash of the package
    pub hash: Option<StringOrArrayOfStrings>,
    /// The installer configuration
    pub installer: Option<Installer>,
    #[serde(flatten)]
    /// The install configuration
    pub install_config: InstallConfig,
}

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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// The install configuration
pub struct InstallConfig {
    pub bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub checkver: Option<Checkver>,
    pub env_add_path: Option<StringOrArrayOfStrings>,
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub extract_dir: Option<StringOrArrayOfStrings>,
    pub hash: Option<StringOrArrayOfStrings>,
    pub installer: Option<Installer>,
    #[deprecated(since = "1.10.0")]
    pub msi: Option<StringOrArrayOfStrings>,
    pub post_install: Option<StringOrArrayOfStrings>,
    pub post_uninstall: Option<StringOrArrayOfStrings>,
    pub pre_install: Option<StringOrArrayOfStrings>,
    pub pre_uninstall: Option<StringOrArrayOfStrings>,
    pub shortcuts: Option<Vec<Vec<String>>>,
    pub uninstaller: Option<Uninstaller>,
    pub url: Option<StringOrArrayOfStrings>,
}

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
    /// Custom PowerShell script to retrieve application version using more complex approach.
    pub script: Option<StringOrArrayOfStrings>,
    pub sourceforge: Option<SourceforgeUnion>,
    pub url: Option<String>,
    pub useragent: Option<String>,
    pub xpath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceforgeClass {
    pub path: Option<String>,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Installer {
    /// Undocumented: only used in scoop-extras/oraclejdk* and scoop-extras/appengine-go
    #[serde(rename = "_comment")]
    pub comment: Option<String>,
    pub args: Option<StringOrArrayOfStrings>,
    pub file: Option<String>,
    pub keep: Option<bool>,
    pub script: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Uninstaller {
    pub args: Option<StringOrArrayOfStrings>,
    pub file: Option<String>,
    pub script: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Autoupdate {
    pub architecture: Option<AutoupdateArchitecture>,
    pub bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub env_add_path: Option<StringOrArrayOfStrings>,
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub extract_dir: Option<StringOrArrayOfStrings>,
    pub hash: Option<HashExtractionOrArrayOfHashExtractions>,
    pub installer: Option<AutoupdateInstaller>,
    pub license: Option<AutoupdateLicense>,
    pub notes: Option<StringOrArrayOfStrings>,
    pub persist: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub psmodule: Option<AutoupdatePsmodule>,
    pub shortcuts: Option<Vec<Vec<String>>>,
    pub url: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdateArchitecture {
    #[serde(rename = "32bit")]
    pub the_32_bit: Option<AutoupdateArch>,
    #[serde(rename = "64bit")]
    pub the_64_bit: Option<AutoupdateArch>,
    pub arm64: Option<AutoupdateArch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdateArch {
    pub bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub env_add_path: Option<StringOrArrayOfStrings>,
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub extract_dir: Option<StringOrArrayOfStrings>,
    pub hash: Option<HashExtractionOrArrayOfHashExtractions>,
    pub installer: Option<PurpleInstaller>,
    pub shortcuts: Option<Vec<Vec<String>>>,
    pub url: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HashExtraction {
    /// Same as 'regex'
    pub find: Option<String>,
    /// Same as 'jsonpath'
    pub jp: Option<String>,
    pub jsonpath: Option<String>,
    pub mode: Option<Mode>,
    pub regex: Option<String>,
    #[deprecated(since = "1.10.0", note = "hash type is determined automatically")]
    #[serde(rename = "type")]
    pub hash_extraction_type: Option<Type>,
    pub url: Option<String>,
    pub xpath: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurpleInstaller {
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdateInstaller {
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct License {
    pub identifier: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AutoupdatePsmodule {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Psmodule {
    pub name: Option<String>,
}

// TODO: Conversions to package reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Suggest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    String(String),
    StringArray(Vec<String>),
    UnionArray(Vec<StringOrArrayOfStrings>),
}

impl StringOrArrayOfStrings {
    #[must_use]
    pub fn into_vec(&self) -> Vec<String> {
        match self {
            StringOrArrayOfStrings::String(s) => vec![s.clone()],
            StringOrArrayOfStrings::StringArray(string_array) => string_array.clone(),
        }
    }
}

impl StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    #[must_use]
    pub fn into_vec(&self) -> Vec<String> {
        match self {
            StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::String(s) => vec![s.clone()],
            StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::StringArray(s) => s.clone(),
            StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::UnionArray(s) => s
                .iter()
                .flat_map(|s| match s {
                    StringOrArrayOfStrings::String(s) => vec![s.clone()],
                    StringOrArrayOfStrings::StringArray(s) => s.clone(),
                })
                .collect(),
        }
    }
}

impl Display for StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_vec().iter().format(", ").fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Checkver {
    CheckverClass(Box<CheckverClass>),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum TOrArrayOfTs<T> {
    T(T),
    Array(Vec<T>),
}

impl<T> TOrArrayOfTs<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            TOrArrayOfTs::T(t) => vec![t],
            TOrArrayOfTs::Array(array) => array,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOfStrings {
    String(String),
    StringArray(Vec<String>),
}

impl Display for StringOrArrayOfStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.into_vec().iter().format(", ").fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SourceforgeUnion {
    SourceforgeClass(SourceforgeClass),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum HashExtractionOrArrayOfHashExtractions {
    Url(String),
    HashExtraction(HashExtraction),
    HashExtractionArray(Vec<HashExtraction>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum AutoupdateLicense {
    License(License),
    String(String),
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mode {
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
}
