//! The install manifest

use serde::{Deserialize, Serialize};

use crate::{
    Architecture,
    contexts::ScoopContext,
    packages::{CreateManifest, Result},
};

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
/// The install manifest
pub struct Manifest {
    /// This must be manually set
    #[serde(skip)]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The bucket the package was installed from
    pub bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether the package is held
    pub hold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The URL the package was installed from
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The architecture of the package
    pub architecture: Option<Architecture>,
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
    /// Get the source of the manifest
    pub fn get_source(&self) -> String {
        match (&self.bucket, &self.url) {
            (Some(bucket), None) => bucket.clone(),
            (None, Some(url)) => url.clone(),
            _ => "Unknown".to_string(),
        }
    }

    /// Get the package manifest from the install manifest
    ///
    /// # Errors
    /// - Missing or invalid manifest
    pub fn get_manifest(&self, ctx: &impl ScoopContext) -> Result<super::manifest::Manifest> {
        let name = unsafe { self.name() };
        let manifest_path = ctx
            .apps_path()
            .join(name)
            .join("current")
            .join("manifest.json");

        Ok(super::manifest::Manifest::from_path(manifest_path)?.with_name(name))
    }
}

#[cfg(test)]
mod tests {
    use super::{Architecture, Manifest};

    #[test]
    fn test_install_manifest_serde() {
        // Formatted the same as serde_json will output
        const MANIFEST: &str = r#"{"bucket":"main","architecture":"64bit"}"#;

        let zig_manifest = Manifest {
            name: None,
            bucket: Some("main".to_string()),
            hold: None,
            url: None,
            architecture: Some(Architecture::X64),
        };

        let deserialized_manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        assert_eq!(deserialized_manifest, zig_manifest);

        let serialized_manifest = serde_json::to_string(&zig_manifest).unwrap();

        assert_eq!(serialized_manifest, MANIFEST);
    }

    #[test]
    fn test_held_install_manifest_serde() {
        // Formatted the same as serde_json will output
        const MANIFEST: &str = r#"{"bucket":"main","hold":true,"architecture":"64bit"}"#;

        let zig_manifest = Manifest {
            name: None,
            bucket: Some("main".to_string()),
            hold: Some(true),
            url: None,
            architecture: Some(Architecture::X64),
        };

        let deserialized_manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        assert_eq!(deserialized_manifest, zig_manifest);

        let serialized_manifest = serde_json::to_string(&zig_manifest).unwrap();

        assert_eq!(serialized_manifest, MANIFEST);
    }
}
