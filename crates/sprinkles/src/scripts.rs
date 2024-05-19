//! Powershell Script helpers
//!
//! This module provides a way to create and run Powershell scripts
//!
//! # Example
//! ```no_run
//! # use sprinkles::{scripts::PowershellScript, contexts::{User, ScoopContext}};
//!
//! let script = PowershellScript::new("Write-Host 'Hello, world!'");
//! # let ctx = User::new();
//! let runner = script.save_to(ctx.scripts_path()).unwrap();
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! runner.run().await.unwrap();
//! # });
//! ```

use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use tokio::process::Command;

use crate::{
    config, contexts::ScoopContext, hash::substitutions::Substitute,
    packages::models::manifest::TOrArrayOfTs,
};

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
    pub fn new(script: impl Into<String>) -> Self {
        Self {
            script: script.into(),
        }
    }

    #[must_use]
    /// Get the script as a string
    pub fn as_str(&self) -> &str {
        &self.script
    }

    /// Save the script to the context's scripts path, and return the path
    ///
    /// The file will be named `<script-hash>.ps1`
    ///
    /// Note that the file will not be overwritten if it already exists.
    /// If you do not plan to re-use the script, you should clean it up yourself.
    ///
    /// # Errors
    /// - The script could not be written to the directory
    pub fn save(&self, ctx: &impl ScoopContext<config::Scoop>) -> Result<ScriptRunner> {
        self.save_to(ctx.scripts_path())
    }

    /// Save the script to a directory, and return the path
    ///
    /// The file will be named `<script-hash>.ps1`
    ///
    /// Note that the file will not be overwritten if it already exists.
    /// If you do not plan to re-use the script, you should clean it up yourself.
    ///
    /// # Errors
    /// - The script could not be written to the directory
    pub fn save_to(&self, directory: impl AsRef<Path>) -> Result<ScriptRunner> {
        let hash = blake3::hash(self.script.as_bytes());

        let file_path = directory.as_ref().join(format!("{hash}.ps1"));

        if !file_path.exists() {
            std::fs::write(&file_path, self.script.as_bytes())?;
        }

        ScriptRunner::from_path(file_path)
    }
}

impl From<String> for PowershellScript {
    fn from(value: String) -> Self {
        Self::new(value)
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

impl Substitute for PowershellScript {
    fn into_substituted(
        mut self,
        params: &crate::hash::substitutions::SubstitutionMap,
        regex_escape: bool,
    ) -> Self {
        self.script.substitute(params, regex_escape);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
/// A script runner
///
/// This is used to run scripts in a powershell environment
pub struct ScriptRunner {
    path: PathBuf,
    powershell_path: PathBuf,
}

impl ScriptRunner {
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

mod ser_de {
    use serde::{Deserialize, Serialize};

    use crate::packages::models::manifest::TOrArrayOfTs;

    use super::PowershellScript;

    impl Serialize for PowershellScript {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let lines = self.script.lines().collect::<Vec<_>>();

            let script_array = TOrArrayOfTs::from_vec(lines);

            script_array.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for PowershellScript {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let script_array = TOrArrayOfTs::<String>::deserialize(deserializer)?;

            Ok(PowershellScript::from(script_array))
        }
    }
}
