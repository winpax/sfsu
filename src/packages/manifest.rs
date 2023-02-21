// Thanks to quicktype.io for saving me a lot of time.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// A comment.
    #[serde(rename = "##")]
    pub(crate) empty: Option<StringOrArrayOfStrings>,
    #[serde(rename = "$schema")]
    pub(crate) schema: Option<String>,
    /// Deprecated. Use ## instead.
    #[serde(rename = "_comment")]
    pub(crate) comment: Option<StringOrArrayOfStrings>,
    pub(crate) architecture: Option<Architecture>,
    pub(crate) autoupdate: Option<Autoupdate>,
    pub(crate) bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub(crate) checkver: Option<Checkver>,
    /// Undocumented: Found at https://github.com/se35710/scoop-java/search?l=JSON&q=cookie
    pub(crate) cookie: Option<HashMap<String, Option<serde_json::Value>>>,
    pub(crate) depends: Option<StringOrArrayOfStrings>,
    pub(crate) description: Option<String>,
    pub(crate) env_add_path: Option<StringOrArrayOfStrings>,
    pub(crate) env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub(crate) extract_dir: Option<StringOrArrayOfStrings>,
    pub(crate) extract_to: Option<StringOrArrayOfStrings>,
    pub(crate) hash: Option<StringOrArrayOfStrings>,
    pub(crate) homepage: Option<String>,
    /// True if the installer InnoSetup based. Found in
    /// https://github.com/ScoopInstaller/Main/search?l=JSON&q=innosetup
    pub(crate) innosetup: Option<bool>,
    pub(crate) installer: Option<Installer>,
    pub(crate) license: Option<PackageLicense>,
    /// Deprecated
    pub(crate) msi: Option<StringOrArrayOfStrings>,
    pub(crate) notes: Option<StringOrArrayOfStrings>,
    pub(crate) persist: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub(crate) post_install: Option<StringOrArrayOfStrings>,
    pub(crate) post_uninstall: Option<StringOrArrayOfStrings>,
    pub(crate) pre_install: Option<StringOrArrayOfStrings>,
    pub(crate) pre_uninstall: Option<StringOrArrayOfStrings>,
    pub(crate) psmodule: Option<Psmodule>,
    pub(crate) shortcuts: Option<Vec<Vec<String>>>,
    pub(crate) suggest: Option<Suggest>,
    pub(crate) uninstaller: Option<Uninstaller>,
    pub(crate) url: Option<StringOrArrayOfStrings>,
    pub(crate) version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Architecture {
    #[serde(rename = "32bit")]
    pub(crate) the_32_bit: Option<The32BitClass>,
    #[serde(rename = "64bit")]
    pub(crate) the_64_bit: Option<The32BitClass>,
    pub(crate) arm64: Option<The32BitClass>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct The32BitClass {
    pub(crate) bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub(crate) checkver: Option<Checkver>,
    pub(crate) env_add_path: Option<StringOrArrayOfStrings>,
    pub(crate) env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub(crate) extract_dir: Option<StringOrArrayOfStrings>,
    pub(crate) hash: Option<StringOrArrayOfStrings>,
    pub(crate) installer: Option<Installer>,
    /// Deprecated
    pub(crate) msi: Option<StringOrArrayOfStrings>,
    pub(crate) post_install: Option<StringOrArrayOfStrings>,
    pub(crate) post_uninstall: Option<StringOrArrayOfStrings>,
    pub(crate) pre_install: Option<StringOrArrayOfStrings>,
    pub(crate) pre_uninstall: Option<StringOrArrayOfStrings>,
    pub(crate) shortcuts: Option<Vec<Vec<String>>>,
    pub(crate) uninstaller: Option<Uninstaller>,
    pub(crate) url: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckverClass {
    pub(crate) github: Option<String>,
    /// Same as 'jsonpath'
    pub(crate) jp: Option<String>,
    pub(crate) jsonpath: Option<String>,
    /// Same as 'regex'
    pub(crate) re: Option<String>,
    pub(crate) regex: Option<String>,
    /// Allows rearrange the regexp matches
    pub(crate) replace: Option<String>,
    /// Reverse the order of regex matches
    pub(crate) reverse: Option<bool>,
    /// Custom PowerShell script to retrieve application version using more complex approach.
    pub(crate) script: Option<StringOrArrayOfStrings>,
    pub(crate) sourceforge: Option<SourceforgeUnion>,
    pub(crate) url: Option<String>,
    pub(crate) useragent: Option<String>,
    pub(crate) xpath: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceforgeClass {
    pub(crate) path: Option<String>,
    pub(crate) project: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Installer {
    /// Undocumented: only used in scoop-extras/oraclejdk* and scoop-extras/appengine-go
    #[serde(rename = "_comment")]
    pub(crate) comment: Option<String>,
    pub(crate) args: Option<StringOrArrayOfStrings>,
    pub(crate) file: Option<String>,
    pub(crate) keep: Option<bool>,
    pub(crate) script: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Uninstaller {
    pub(crate) args: Option<StringOrArrayOfStrings>,
    pub(crate) file: Option<String>,
    pub(crate) script: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Autoupdate {
    pub(crate) architecture: Option<AutoupdateArchitecture>,
    pub(crate) bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub(crate) env_add_path: Option<StringOrArrayOfStrings>,
    pub(crate) env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub(crate) extract_dir: Option<StringOrArrayOfStrings>,
    pub(crate) hash: Option<HashExtractionOrArrayOfHashExtractions>,
    pub(crate) installer: Option<AutoupdateInstaller>,
    pub(crate) license: Option<AutoupdateLicense>,
    pub(crate) notes: Option<StringOrArrayOfStrings>,
    pub(crate) persist: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub(crate) psmodule: Option<AutoupdatePsmodule>,
    pub(crate) shortcuts: Option<Vec<Vec<String>>>,
    pub(crate) url: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoupdateArchitecture {
    #[serde(rename = "32bit")]
    pub(crate) the_32_bit: Option<AutoupdateArch>,
    #[serde(rename = "64bit")]
    pub(crate) the_64_bit: Option<AutoupdateArch>,
    pub(crate) arm64: Option<AutoupdateArch>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoupdateArch {
    pub(crate) bin: Option<StringOrArrayOfStringsOrAnArrayOfArrayOfStrings>,
    pub(crate) env_add_path: Option<StringOrArrayOfStrings>,
    pub(crate) env_set: Option<HashMap<String, Option<serde_json::Value>>>,
    pub(crate) extract_dir: Option<StringOrArrayOfStrings>,
    pub(crate) hash: Option<HashExtractionOrArrayOfHashExtractions>,
    pub(crate) installer: Option<PurpleInstaller>,
    pub(crate) shortcuts: Option<Vec<Vec<String>>>,
    pub(crate) url: Option<StringOrArrayOfStrings>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashExtraction {
    /// Same as 'regex'
    pub(crate) find: Option<String>,
    /// Same as 'jsonpath'
    pub(crate) jp: Option<String>,
    pub(crate) jsonpath: Option<String>,
    pub(crate) mode: Option<Mode>,
    pub(crate) regex: Option<String>,
    /// Deprecated, hash type is determined automatically
    #[serde(rename = "type")]
    pub(crate) hash_extraction_type: Option<Type>,
    pub(crate) url: Option<String>,
    pub(crate) xpath: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurpleInstaller {
    pub(crate) file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoupdateInstaller {
    pub(crate) file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub(crate) identifier: String,
    pub(crate) url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoupdatePsmodule {
    pub(crate) name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Psmodule {
    pub(crate) name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Suggest {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArrayOfStringsOrAnArrayOfArrayOfStrings {
    String(String),
    UnionArray(Vec<StringOrArrayOfStringsElement>),
}

/// A comment.
///
/// Deprecated. Use ## instead.
///
/// Custom `PowerShell` script to retrieve application version using more complex approach.
///
/// Deprecated
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArrayOfStringsElement {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArrayOfStrings {
    String(String),
    StringArray(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceforgeUnion {
    SourceforgeClass(SourceforgeClass),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HashExtractionOrArrayOfHashExtractions {
    HashExtraction(HashExtraction),
    HashExtractionArray(Vec<HashExtraction>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AutoupdateLicense {
    License(License),
    String(String),
}

#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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
