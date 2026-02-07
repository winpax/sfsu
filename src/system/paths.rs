use std::path::PathBuf;

use quork::traits::truthy::ContainsTruth;

#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
#[allow(clippy::enum_variant_names, dead_code)]
/// This is a non-exhaustive list CSIDLs for Windows defined paths
pub enum Paths {
    /// Persistent application data for the current user
    AppData,
    /// Non-persistent application data for the current user
    LocalAppData,
    /// Persistent application data for all users
    CommonAppData,
}

impl Paths {
    pub fn into_path(self) -> Option<PathBuf> {
        match self {
            Self::AppData => dirs::data_dir(),
            Self::LocalAppData => dirs::data_local_dir(),
            Paths::CommonAppData => {
                let path = PathBuf::from("C:\\ProgramData");
                path.try_exists().contains_truth().then_some(path)
            }
        }
    }
}
