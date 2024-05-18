//! Package handles

use std::path::PathBuf;

use crate::packages::reference::package;

pub struct PackageHandle {
    reference: package::Reference,
    path: PathBuf,
}
