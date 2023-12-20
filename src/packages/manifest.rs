// Thanks to quicktype.io for saving me a lot of time.
// The names are a bit weird at times but I'll work on that in future.

use std::{collections::HashMap, fmt::Display};

use itertools::Itertools as _;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
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
    pub schema: Option<String>,
    /// Deprecated. Use ## instead.
    #[serde(rename = "_comment")]
    pub comment: Option<StringOrArrayOfStrings>,
    pub architecture: Option<Architecture>,
    pub autoupdate: Option<Autoupdate>,
    pub bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub checkver: Option<Checkver>,
    /// Undocumented: Found at https://github.com/se35710/scoop-java/search?l=JSON&q=cookie
    pub cookie: Option<HashMap<String, Option<serde_json::Value>>>,
    pub depends: Option<StringOrArrayOfStrings>,
    pub description: Option<String>,
    pub env_add_path: Option<StringOrArrayOfStrings>,
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub extract_dir: Option<StringOrArrayOfStrings>,
    pub extract_to: Option<StringOrArrayOfStrings>,
    pub hash: Option<StringOrArrayOfStrings>,
    pub homepage: Option<String>,
    /// True if the installer InnoSetup based. Found in
    /// https://github.com/ScoopInstaller/Main/search?l=JSON&q=innosetup
    pub innosetup: Option<bool>,
    pub installer: Option<Installer>,
    pub license: Option<PackageLicense>,
    #[deprecated]
    /// Deprecated
    pub msi: Option<StringOrArrayOfStrings>,
    pub notes: Option<StringOrArrayOfStrings>,
    pub persist: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub post_install: Option<StringOrArrayOfStrings>,
    pub post_uninstall: Option<StringOrArrayOfStrings>,
    pub pre_install: Option<StringOrArrayOfStrings>,
    pub pre_uninstall: Option<StringOrArrayOfStrings>,
    pub psmodule: Option<Psmodule>,
    pub shortcuts: Option<Vec<Vec<String>>>,
    pub suggest: Option<Suggest>,
    pub uninstaller: Option<Uninstaller>,
    pub url: Option<StringOrArrayOfStrings>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Architecture {
    #[serde(rename = "32bit")]
    pub the_32_bit: Option<The32BitClass>,
    #[serde(rename = "64bit")]
    pub the_64_bit: Option<The32BitClass>,
    pub arm64: Option<The32BitClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct The32BitClass {
    pub bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub checkver: Option<Checkver>,
    pub env_add_path: Option<StringOrArrayOfStrings>,
    pub env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub extract_dir: Option<StringOrArrayOfStrings>,
    pub hash: Option<StringOrArrayOfStrings>,
    pub installer: Option<Installer>,
    /// Deprecated
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
    /// Deprecated, hash type is determined automatically
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Suggest {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    String(String),
    StringArray(Vec<String>),
    UnionArray(Vec<StringOrArrayOfStringsElement>),
}

impl StringOrArrayOfStrings {
    #[must_use]
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            StringOrArrayOfStrings::String(s) => vec![s.clone()],
            StringOrArrayOfStrings::StringArray(string_array) => string_array.clone(),
        }
    }
}

impl StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    #[must_use]
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::String(s) => vec![s.clone()],
            StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::StringArray(s) => s.clone(),
            StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::UnionArray(s) => s
                .iter()
                .flat_map(|s| match s {
                    StringOrArrayOfStringsElement::String(s) => vec![s.clone()],
                    StringOrArrayOfStringsElement::StringArray(s) => s.clone(),
                })
                .collect(),
        }
    }
}

impl Display for StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_vec().iter().format(", ").fmt(f)
    }
}

/// A comment.
///
/// Deprecated. Use ## instead.
///
/// Custom `PowerShell` script to retrieve application version using more complex approach.
///
/// Deprecated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOfStringsElement {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Checkver {
    CheckverClass(Box<CheckverClass>),
    String(String),
}

/// A comment.
///
/// Deprecated. Use ## instead.
///
/// Custom `PowerShell` script to retrieve application version using more complex approach.
///
/// Deprecated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StringOrArrayOfStrings {
    String(String),
    StringArray(Vec<String>),
}

impl Display for StringOrArrayOfStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_vec().iter().format(", ").fmt(f)
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
        }
    }
}

/// Deprecated, hash type is determined automatically
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
