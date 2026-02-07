//! Utilities for running installers and uninstallers

use crate::contexts::ScoopContext;
use crate::handles::packages::PackageHandle;
use crate::hash::substitutions::{Substitute, SubstitutionMap};
use crate::hash::url_ext::UrlExt;
use crate::packages::models::manifest::SingleOrArray;
use crate::scripts::summary::Summary;
use crate::{Architecture, packages::models::manifest::Installer};
use quork::prelude::ContainsTruth;
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Errors that can occur when running an installer
pub enum Error {
    #[error("Invalid installer. No file name or urls were provided")]
    MissingFileName,
    #[error("The uninstall program is not located in the version directory")]
    ProgramOutsideVersionDir,
    #[error("The uninstall program could not be found")]
    ProgramNotFound,
    #[error("Invalid url provided in manifest: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Could not open the package handle: {0}")]
    HandleError(#[from] crate::handles::packages::Error),
    #[error("Could not run the powershell script: {0}")]
    PowershellError(#[from] super::Error),
    #[error("Could not invoke the uninstaller: {0}")]
    IO(#[from] std::io::Error),
    #[error("Uninstaller exited with code {0}")]
    Uninstaller(std::process::ExitStatus),
}

#[allow(missing_docs)]
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[must_use]
/// Controller for the execution of installers
pub struct Runner<'a, 'c, C> {
    handle: &'a PackageHandle<'c, C>,
    installer: Installer,
    architecture: Architecture,
}

impl<'a, 'c, C: ScoopContext> Runner<'a, 'c, C> {
    /// Construct a new installer runner from an installer
    pub fn new(handle: &'a PackageHandle<'c, C>, installer: impl Into<Installer>) -> Self {
        Self {
            handle,
            installer: installer.into(),
            architecture: Architecture::ARCH,
        }
    }

    /// Provide a specific architecture to the installer runner
    pub fn with_architecture(self, architecture: Architecture) -> Self {
        Self {
            architecture,
            ..self
        }
    }

    /// Get a summary of what will be run for this installer
    ///
    /// # Errors
    /// - Could not determine the program name
    /// - Could not create a [`PowershellScript`] from the program name
    pub fn get_summary(&self) -> Result<Summary> {
        let prog_name = self.prog_name()?;
        let args = self.subbed_args();

        Ok(if self.is_powershell() {
            Summary::Powershell {
                script: super::PowershellScript::from_path(prog_name)?,
                args,
            }
        } else {
            Summary::Command {
                command: prog_name,
                args,
            }
        })
    }

    pub(crate) fn prog_name(&self) -> Result<PathBuf> {
        let installer = &self.installer;
        let manifest = self.handle.remote_manifest();

        let name = if let Some(name) = &installer.file {
            name.clone()
        } else {
            let install_config = manifest.install_config(self.architecture);

            if let Some(urls) = install_config.urls {
                let mut urls = urls.iter();
                let first_url = urls.next();

                if let Some(first_url) = first_url {
                    Url::parse(first_url)?.remote_filename()
                } else {
                    return Err(Error::MissingFileName);
                }
            } else {
                return Err(Error::MissingFileName);
            }
        };

        let version_dir = self.handle.version_dir();

        Ok(dunce::canonicalize(version_dir.join(name))?)
    }

    pub(crate) fn substitutions(&self) -> SubstitutionMap {
        let mut map = HashMap::new();
        map.insert(
            "$dir",
            self.handle.version_dir().to_string_lossy().to_string(),
        );
        map.insert("$global", (C::CONTEXT_NAME == "global").to_string());
        map.insert(
            "$version",
            self.handle.remote_manifest().version.to_string(),
        );

        SubstitutionMap::from(map)
    }

    pub(crate) fn subbed_args(&self) -> Vec<String> {
        let substitutions = self.substitutions();

        self.installer
            .args
            .clone()
            .map(|args| args.into_substituted(&substitutions, false))
            .map(SingleOrArray::to_vec)
            .unwrap_or_default()
    }

    pub(crate) fn is_powershell(&self) -> bool {
        self.prog_name()
            .is_ok_and(|path| path.extension() == Some(std::ffi::OsStr::new("ps1")))
    }

    /// Run the installer
    ///
    /// Note that this does not run the 'uninstall' hook script.
    /// It is expected that you run the hook script yourself.
    ///
    ///
    /// # Errors
    /// - The uninstaller could not be found
    /// - The uninstaller could not be run
    /// - The uninstaller exited with a non-zero exit code
    /// - The uninstaller is outside the version directory
    /// - Failed to invoke the uninstaller
    /// - The manifest install config had neither a file name nor urls
    /// - The url provided was invalid
    ///
    /// For more information on errors, see [`Error`]
    pub fn run(self, ctx: &impl ScoopContext) -> Result<()> {
        let installer = &self.installer;

        if installer.file.is_some() || installer.args.is_some() {
            let version_dir = self.handle.version_dir();
            let prog_name = self.prog_name()?;

            if !prog_name.starts_with(&version_dir) {
                return Err(Error::ProgramOutsideVersionDir);
            } else if !prog_name.exists() {
                return Err(Error::ProgramNotFound);
            }

            let args = self.subbed_args();

            if self.is_powershell() {
                let script = super::PowershellScript::from_path(prog_name)?;
                let mut runner = script.save(ctx)?;
                runner.set_args(args);
                runner.run()?;
            } else {
                let mut cmd = std::process::Command::new(&prog_name);

                cmd.args(args);

                let output = cmd.output()?;

                if !output.status.success() {
                    return Err(Error::Uninstaller(output.status));
                }

                if !installer.keep.contains_truth() {
                    std::fs::remove_file(prog_name)?;
                }
            }
        }

        Ok(())
    }
}
