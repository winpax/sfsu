//! Package handles

use std::{path::PathBuf, rc::Rc};

use crate::{
    contexts::ScoopContext,
    packages::{
        CreateManifest, InstallManifest, Manifest,
        reference::{self, manifest, package},
    },
    system::common::{Common, System},
    version::Version,
};

use super::version::VersionHandle;

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
    #[error("Version handle error: {0}")]
    VersionHandle(#[from] super::version::Error),
    #[error("Unsupported manifest reference. The manifest must be a local file")]
    UnsupportedManifestReference,
    #[error("Package not installed")]
    PackageNotInstalled,
    #[error("Package was not installed correctly")]
    BrokenInstall,
    #[error("Manifest was not provided a bucket")]
    MissingBucket,
    #[error("Manifest was not provided a name")]
    MissingName,
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

impl<'a, C: ScoopContext> PackageHandle<'a, C> {
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

    /// Create a new package handle from a manifest
    ///
    /// # Errors
    /// - The manifest was not provided a bucket
    /// - The manifest was not provided a name
    /// - Any further errors from [`PackageHandle::new`]
    pub async fn from_manifest(ctx: &'a C, manifest: &Manifest) -> Result<Self> {
        let reference = manifest::Reference::BucketNamePair {
            bucket: manifest
                .bucket_opt()
                .ok_or(Error::MissingBucket)?
                .to_string(),
            name: manifest.name_opt().ok_or(Error::MissingName)?.to_string(),
        };

        Self::new(ctx, reference.into()).await
    }

    #[must_use]
    /// Get the package's install path
    ///
    /// This will return either the `current` folder, or the version directory,
    /// if the `current` folder is not linked or the config has `no_junction` set to `true`
    pub fn current(&self) -> PathBuf {
        let current = self.path.join("current");

        if !current.exists() || !self.ctx.symlinks_enabled() {
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
        self.ctx.persist_path().join(unsafe { self.name() })
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

    /// Get the package's local version
    ///
    /// # Errors
    /// - See more at [`PackageHandle::local_manifest`]
    pub fn local_version(&self) -> Result<Version> {
        self.local_manifest().map(|manifest| manifest.version)
    }

    /// List all versions of the package
    ///
    /// # Errors
    /// - Reading the package's versions failed
    pub fn list_versions(&self) -> Result<Vec<VersionHandle>, Error> {
        let mut versions = Vec::new();

        for entry in std::fs::read_dir(&self.path)? {
            let path = entry?.path();

            if let Ok(version) = VersionHandle::try_from(path) {
                versions.push(version);
            }
        }

        Ok(versions)
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
        if !self.ctx.symlinks_enabled() {
            return Ok(());
        }

        self.unlink_current()?;

        let current_path = self.path.join("current");
        let version_dir = self.version_dir();

        System::symlink_dir(version_dir, current_path)?;

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
    ///
    /// # Safety
    /// This field is manually set in the remote manifest, and by default is uninitialized. This may cause undefined behavior.
    ///
    /// Use [`Manifest::name_opt`] or,
    /// to ensure that this function returns properly, use the [`CreateManifest`] trait to set the name,
    /// or create the manifest, rather than other methods that might fail to set the name.
    pub unsafe fn name(&self) -> &str {
        unsafe { self.remote_manifest.name() }
    }

    #[must_use]
    /// Get the package's reference
    pub fn reference(&self) -> &package::Reference {
        self.as_ref()
    }

    #[must_use]
    /// Check if the package handle owns a running process
    pub fn running(&self) -> bool {
        #[cfg(not(windows))]
        windows_only!();

        #[cfg(windows)]
        {
            use crate::system::process;

            let process_dir = self.version_dir();

            unsafe { process::Process::BaseDir(process_dir).find_running() }.unwrap_or(false)
        }
    }
}

impl<C> AsRef<package::Reference> for PackageHandle<'_, C> {
    fn as_ref(&self) -> &package::Reference {
        &self.reference
    }
}
