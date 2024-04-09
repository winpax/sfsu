use serde::Serialize;

use crate::{
    output::wrappers::{alias_vec::AliasVec, bool::NicerBool},
    packages::manifest::PackageLicense,
};

#[derive(Debug, Clone, Serialize)]
pub struct PackageInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub version: String,
    pub bucket: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<PackageLicense>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    pub installed: NicerBool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binaries: Option<String>,
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shortcuts: Option<AliasVec<String>>,
}
