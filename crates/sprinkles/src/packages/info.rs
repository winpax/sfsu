use serde::Serialize;

use crate::{
    output::wrappers::{alias_vec::AliasVec, bool::NicerBool},
    packages::manifest::PackageLicense,
};

#[derive(Debug, Clone, Serialize, sfsu_derive::KeyValue)]
pub struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub bucket: String,
    pub website: Option<String>,
    pub license: Option<PackageLicense>,
    pub updated_at: Option<String>,
    pub updated_by: Option<String>,
    pub installed: NicerBool,
    pub binaries: Option<String>,
    pub notes: Option<String>,
    pub shortcuts: Option<AliasVec<String>>,
}
