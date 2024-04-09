//! Summary info for a package

use serde::Serialize;

use crate::{
    output::wrappers::{alias_vec::AliasVec, bool::NicerBool},
    packages::manifest::PackageLicense,
};

#[derive(Debug, Clone, Serialize)]
/// Summary package information
pub struct PackageInfo {
    /// The name of the package
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The description of the package
    pub description: Option<String>,
    /// The version of the package
    pub version: String,
    /// The bucket the package is in
    pub bucket: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The homepage of the package
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The license of the package
    pub license: Option<PackageLicense>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The last time the package was updated
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The last time the package was updated by
    pub updated_by: Option<String>,
    /// Whether the package is installed
    pub installed: NicerBool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The list of the package's binaries
    pub binaries: Option<String>,
    /// The package's notes
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The package's shortcuts
    pub shortcuts: Option<AliasVec<String>>,
}
