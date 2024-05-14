//! Powershell Script helpers
//!
//! This module provides a way to create and run Powershell scripts
//!
//! # Example
//! ```
//! use sprinkles::scripts::PowershellScript;
//!
//! let script = PowershellScript::new("Write-Host 'Hello, world!'");
//!
//! let path = script.save_to("C:\\Users\\me\\Desktop").unwrap();
//!
//! let output = script.run().unwrap();
//! ```

use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use tokio::process::Command;

use crate::packages::models::manifest::TOrArrayOfTs;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Errors that can occur when running a script
pub enum Error {
    #[error("Powershell exited with code {0}")]
    PowershellExit(ExitStatus),
    #[error("Could not find powershell in path")]
    FindPowershell(#[from] which::Error),
    #[error("Running script: {0}")]
    IOError(#[from] std::io::Error),
}

/// A Powershell script runner result
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A powershell script
///
/// These are used for ``pre_install``, ``post_install``, etc. scripts from the manifest
pub struct PowershellScript {
    script: String,
}

impl PowershellScript {
    #[must_use]
    /// Create a new powershell script
    pub fn new(script: impl Into<Self>) -> Self {
        script.into()
    }

    #[must_use]
    /// Get the script as a string
    pub fn as_str(&self) -> &str {
        &self.script
    }

    /// Save the script to a directory, and return the path
    ///
    /// The file will be named `<script-hash>.ps1`
    ///
    /// # Errors
    /// - The script could not be written to the directory
    pub fn save_to(&self, directory: &Path) -> Result<ScriptRunner> {
        let hash = blake3::hash(self.script.as_bytes());

        let file_path = directory.join(format!("{hash}.ps1"));

        std::fs::write(&file_path, self.script.as_bytes())?;

        ScriptRunner::from_path(file_path)
    }
}

impl From<String> for PowershellScript {
    fn from(value: String) -> Self {
        Self { script: value }
    }
}

impl From<PowershellScript> for String {
    fn from(value: PowershellScript) -> Self {
        value.script
    }
}

impl From<TOrArrayOfTs<String>> for PowershellScript {
    fn from(value: TOrArrayOfTs<String>) -> Self {
        match value {
            TOrArrayOfTs::Single(s) => Self::from(s),
            TOrArrayOfTs::Array(array) => Self::from(array.join("\n")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A script runner
///
/// This is used to run scripts in a powershell environment
pub struct ScriptRunner {
    path: PathBuf,
    powershell_path: PathBuf,
}

impl ScriptRunner {
    #[must_use]
    /// Create a new script runner
    pub fn new(path: impl AsRef<Path>, powershell_path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let powershell_path = powershell_path.as_ref().to_path_buf();

        Self {
            path,
            powershell_path,
        }
    }

    /// Create a new script runner, getting powershell from the system path
    ///
    /// # Errors
    /// - If powershell is not found in the system path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let powershell_path = which::which("pwsh").or_else(|_| which::which("powershell"))?;

        Ok(Self {
            path,
            powershell_path,
        })
    }

    /// Run a script
    ///
    /// # Errors
    /// - If powershell exited with a non-zero exit code
    /// - If powershell could not be found in the system path
    /// - If the script could not be written to the path
    pub async fn run(&self) -> Result<()> {
        let output = Command::new(&self.powershell_path)
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&self.path)
            .output()
            .await?;

        if !output.status.success() {
            return Err(Error::PowershellExit(output.status));
        }

        Ok(())
    }
}
