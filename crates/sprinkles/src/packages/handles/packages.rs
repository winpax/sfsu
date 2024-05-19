//! Package handles

use std::{path::PathBuf, rc::Rc};

use crate::{
    config,
    contexts::ScoopContext,
    packages::{
        reference::{self, package},
        CreateManifest, InstallManifest, Manifest,
    },
};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Package handle errors
pub enum Error {
    #[error("Package reference error: {0}")]
    ReferenceError(#[from] reference::Error),
    #[error("Package manifest error: {0}")]
    PackagesError(#[from] crate::packages::Error),
    #[error("Linking/unlinking current failed: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Unsupported manifest reference. The manifest must be a local file")]
    UnsupportedManifestReference,
    #[error("Package not installed")]
    PackageNotInstalled,
    #[error("Package was not installed correctly")]
    BrokenInstall,
}

/// Package handle result type
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[must_use]
/// A package handle
pub struct PackageHandle<'a, C> {
    ctx: &'a C,
    reference: package::Reference,
    remote_manifest: Manifest,
    path: PathBuf,
}

impl<'a, C: ScoopContext<config::Scoop>> PackageHandle<'a, C> {
    /// Create a new package handle
    ///
    /// # Errors
    /// - The package reference is invalid
    /// - The package is not installed
    /// - The package was not installed correctly
    /// - The package's remote manifest could not be found or parsed
    /// - The package's install directory could not be found
    pub async fn new(ctx: &'a C, reference: package::Reference) -> Result<Self> {
        let name = reference
            .name()
            .ok_or(Error::UnsupportedManifestReference)?;

        if !reference.installed(ctx)? {
            return Err(Error::PackageNotInstalled);
        }

        let apps_path = ctx.apps_path();
        let path = apps_path.join(name);

        if !path.exists() {
            return Err(Error::BrokenInstall);
        }

        let remote_manifest = reference.manifest(ctx).await?;

        Ok(Self {
            ctx,
            reference,
            remote_manifest,
            path,
        })
    }

    #[must_use]
    /// Get the package's install path
    ///
    /// This will return either the `current` folder, or the version directory,
    /// if the `current` folder is not linked or the config has `no_junction` set to `true`
    pub fn current(&self) -> PathBuf {
        let current = self.path.join("current");

        if !current.exists() || self.ctx.config().no_junction {
            self.version_dir()
        } else {
            current
        }
    }

    #[must_use]
    /// Get the package's remote manifest
    pub fn remote_manifest(&self) -> &Manifest {
        &self.remote_manifest
    }

    /// Get the package's manifest
    ///
    /// # Errors
    /// - Loading and parsing the manifest failed
    pub fn local_manifest(&self) -> Result<Manifest> {
        let manifest_path = self.current().join("manifest.json");

        Ok(Manifest::from_path(manifest_path)?)
    }

    /// Get the package's install manifest
    ///
    /// # Errors
    /// - Loading and parsing the install manifest failed
    pub fn install_manifest(&self) -> Result<InstallManifest> {
        let install_path = self.current().join("install.json");

        Ok(InstallManifest::from_path(install_path)?)
    }

    #[must_use]
    /// Get the package's persist directory
    pub fn persist_dir(&self) -> PathBuf {
        self.ctx.persist_path().join(self.name())
    }

    #[must_use]
    /// Get the package's current version directory
    ///
    /// This will return the version of the remote manifest,
    /// or the version of the package reference, if specified
    pub fn version_dir(&self) -> PathBuf {
        let version = if let Some(ref version) = self.reference.version {
            version
        } else {
            self.remote_manifest.version.as_str()
        };

        self.path.join(version)
    }

    /// Unlink the current folder
    ///
    /// # Errors
    /// - Unlinking the current folder failed
    pub fn unlink_current(&self) -> Result<()> {
        let current_path = self.path.join("current");

        if current_path.exists() {
            std::fs::remove_dir_all(current_path)?;
        }

        Ok(())
    }

    /// Link the current folder to the package's install folder
    ///
    /// This will do nothing if the config has `no_junction` set to `true`
    ///
    /// # Errors
    /// - Unlinking the current folder failed
    /// - Linking the current folder failed
    /// - The current folder is not a symlink
    pub fn link_current(&self) -> Result<()> {
        use std::os::windows::fs;

        if self.ctx.config().no_junction {
            return Ok(());
        }

        self.unlink_current()?;

        let current_path = self.path.join("current");
        let version_dir = self.version_dir();

        fs::symlink_dir(version_dir, current_path)?;

        Ok(())
    }

    /// Get the package's version paths
    ///
    /// # Errors
    /// - Reading the package's version paths failed
    pub fn version_paths(&self) -> Result<Rc<[PathBuf]>> {
        self.path
            .read_dir()?
            .map(|entry| entry.map(|e| e.path()).map_err(Error::from))
            .collect()
    }

    #[must_use]
    /// Get the package's name
    pub fn name(&self) -> &str {
        &self.remote_manifest.name
    }

    #[must_use]
    /// Get the package's reference
    pub fn reference(&self) -> &package::Reference {
        self.as_ref()
    }
}

impl<C> AsRef<package::Reference> for PackageHandle<'_, C> {
    fn as_ref(&self) -> &package::Reference {
        &self.reference
    }
}
