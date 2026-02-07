//! Summaries of what will run for a hook or installer

use std::path::PathBuf;

use crate::{contexts::ScoopContext, scripts::PowershellScript};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A summary of what will run for a hook or installer
pub enum Summary {
    /// A command will be run
    Command {
        /// The command (executable) to run
        command: PathBuf,
        /// The arguments to be passed to the command
        args: Vec<String>,
    },
    /// A powershell script will be run
    Powershell {
        /// The powershell script to run
        script: PowershellScript,
        /// The arguments to be passed to the script
        args: Vec<String>,
    },
}

impl Summary {
    /// Construct a new summary from a value
    ///
    /// # Errors
    /// - The value could not be converted to a summary (see implementations for more information)
    pub fn new<T: TryInto<Summary>>(value: T) -> Result<Self, T::Error> {
        value.try_into()
    }
}

impl From<PowershellScript> for Summary {
    fn from(value: PowershellScript) -> Self {
        Self::Powershell {
            script: value,
            args: vec![],
        }
    }
}

impl<'a, 'c, C: ScoopContext> TryFrom<super::installer::Runner<'a, 'c, C>> for Summary {
    type Error = super::installer::Error;

    fn try_from(value: super::installer::Runner<'a, 'c, C>) -> Result<Self, Self::Error> {
        value.get_summary()
    }
}
