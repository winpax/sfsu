//! Summary info for a package

use chrono::Local;
use serde::Serialize;

use sprinkles::{
    git::parity::SignatureDisplay,
    packages::models::manifest::{AliasArray, PackageLicense},
};

use crate::wrappers::{bool::NicerBool, serialize::SerializeDisplay, time::NicerTime};

#[derive(Clone, Serialize)]
/// Summary package information
pub struct Package<'manifest> {
    /// The name of the package
    pub name: &'manifest str,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The description of the package
    pub description: Option<&'manifest str>,
    /// The version of the package
    pub version: &'manifest str,
    /// The bucket the package is in
    pub bucket: &'manifest str,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The homepage of the package
    pub website: Option<&'manifest str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The license of the package
    pub license: Option<PackageLicense>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The last time the package was updated
    pub updated_at: Option<NicerTime<Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The last time the package was updated by
    pub updated_by: Option<SerializeDisplay<SignatureDisplay<'manifest>>>,
    /// Whether the package is installed
    pub installed: NicerBool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The list of the package's binaries
    pub binaries: Option<&'manifest str>,
    /// The package's notes
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The package's shortcuts
    pub shortcuts: Option<SerializeDisplay<AliasArray<String>>>,
}
