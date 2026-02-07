//! Powershell Script helpers
//!
//! This module provides a way to create and run Powershell scripts
//!
//! # Example
//! ```no_run
//! # use sprinkles::{scripts::PowershellScript, contexts::{User, ScoopContext}};
//!
//! let script = PowershellScript::new("Write-Host 'Hello, world!'");
//! # let ctx = User::new().unwrap();
//! let runner = script.save_to(ctx.scripts_path()).unwrap();
//! runner.run().unwrap();
//! ```

use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
};

use summary::Summary;

use crate::{contexts::ScoopContext, packages::models::manifest::SingleOrArray};

pub mod installer;
pub mod summary;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Errors that can occur when running a script
pub enum Error {
    #[error("Powershell exited with code {0}")]
    PowershellExit(ExitStatus, Output),
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

    /// Get a summary of what will run from the [`PowershellScript`]
    ///
    /// This will clone the [`PowershellScript`].
    /// To get the summary without cloning use [`PowershellScript::into_summary`]
    #[must_use]
    pub fn get_summary(&self) -> Summary {
        self.clone().into_summary()
    }
    /// Get a summary of what will run from the [`PowershellScript`]
    #[must_use]
    pub fn into_summary(self) -> Summary {
        Summary::from(self)
    }

    /// Create a new powershell script from a file
    ///
    /// # Errors
    /// - Reading the file failed
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)?;

        Ok(Self::new(contents))
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
    pub fn save(&self, ctx: &impl ScoopContext) -> Result<ScriptRunner> {
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

impl From<SingleOrArray<String>> for PowershellScript {
    fn from(value: SingleOrArray<String>) -> Self {
        match value {
            SingleOrArray::Single(s) => Self::from(s),
            SingleOrArray::Array(array) => Self::from(array.join("\n")),
        }
    }
}

#[cfg(feature = "manifest-hashes")]
impl crate::hash::substitutions::Substitute for PowershellScript {
    fn substitute(
        &mut self,
        params: &crate::hash::substitutions::SubstitutionMap,
        regex_escape: bool,
    ) {
        self.script.substitute(params, regex_escape);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
/// A script runner
///
/// This is used to run scripts in a powershell environment
pub struct ScriptRunner {
    args: Vec<String>,
    path: PathBuf,
    powershell_path: PathBuf,
}

impl ScriptRunner {
    /// Create a new script runner
    pub fn new(path: impl AsRef<Path>, powershell_path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();
        let powershell_path = powershell_path.as_ref().to_path_buf();

        Self {
            args: vec![],
            path,
            powershell_path,
        }
    }

    /// Set the arguments to pass to the script
    pub fn set_args(&mut self, args: Vec<String>) {
        self.args = args;
    }

    /// Create a new script runner, getting powershell from the system path
    ///
    /// # Errors
    /// - If powershell is not found in the system path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let powershell_path = which::which("pwsh").or_else(|_| which::which("powershell"))?;

        Ok(Self {
            args: vec![],
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
    pub fn run(&self) -> Result<Output> {
        let output = Command::new(&self.powershell_path)
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(&self.path)
            .args(&self.args)
            .output()?;

        if !output.status.success() {
            return Err(Error::PowershellExit(output.status, output));
        }

        Ok(output)
    }
}

mod ser_de {
    use serde::{Deserialize, Serialize};

    use crate::packages::models::manifest::SingleOrArray;

    use super::PowershellScript;

    impl Serialize for PowershellScript {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let lines = self.script.lines().collect::<Vec<_>>();

            let script_array = SingleOrArray::from_vec(lines);

            script_array.serialize(serializer)
        }
    }

    impl<'de> Deserialize<'de> for PowershellScript {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let script_array = SingleOrArray::<String>::deserialize(deserializer)?;

            Ok(PowershellScript::from(script_array))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::contexts::User;

    use super::*;

    #[test]
    fn test_powershell_hello_world() {
        let ctx = User::new().unwrap();

        let script = PowershellScript::new("Write-Host 'Hello, world!'")
            .save(&ctx)
            .unwrap();

        let output = script.run().unwrap();

        assert_eq!(output.status.code(), Some(0));
        assert_eq!(output.stdout, b"Hello, world!\r\n");
    }
}
