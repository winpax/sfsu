pub mod bucket;
pub mod depends;
pub mod describe;
pub mod hook;
pub mod info;
pub mod list;
pub mod search;
pub mod status;
pub mod update;

pub mod home;
#[cfg(not(feature = "v2"))]
pub mod outdated;

use clap::Subcommand;

use sfsu_derive::{Hooks, Runnable};

pub struct DeprecationWarning {
    /// Deprecation message
    message: DeprecationMessage,
    /// Version to be removed in
    version: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum DeprecationMessage {
    /// Replacement info
    Replacement(&'static str),
    /// Warning message
    Warning(&'static str),
}

// TODO: Run command could return `impl Display` and print that itself
pub trait Command {
    fn deprecated() -> Option<DeprecationWarning> {
        None
    }

    fn runner(self) -> Result<(), anyhow::Error>;

    fn run(self) -> Result<(), anyhow::Error>
    where
        Self: Sized,
    {
        if let Some(deprecation_warning) = Self::deprecated() {
            use colored::Colorize as _;

            let mut output = String::from("DEPRECATED: ");

            match deprecation_warning.message {
                DeprecationMessage::Replacement(replacement) => {
                    output += &format!("Use `{replacement}` instead. ");
                }
                DeprecationMessage::Warning(warning) => output += &warning,
            }

            if let Some(version) = deprecation_warning.version {
                output += &format!("Will be removed in v{version}. ");
            }

            println!("{}\n", output.yellow());
        }

        self.runner()
    }
}

#[derive(Debug, Clone, Subcommand, Hooks, Runnable)]
pub enum Commands {
    /// Search for a package
    Search(search::Args),
    /// List all installed packages
    List(list::Args),
    /// Generate hooks for the given shell
    Hook(hook::Args),
    #[cfg(not(feature = "v2"))]
    /// Find buckets that do not have any installed packages
    UnusedBuckets(bucket::unused::Args),
    #[cfg_attr(not(feature = "v2"), no_hook)]
    /// Manages buckets
    Bucket(bucket::Args),
    #[cfg(not(feature = "v2"))]
    /// Describe a package
    Describe(describe::Args),
    /// Display information about a package
    Info(info::Args),
    #[cfg(not(feature = "v2"))]
    /// List outdated buckets and/or packages
    Outdated(outdated::Args),
    /// List the dependencies of a given package, in the order that they will be installed
    Depends(depends::Args),
    /// Show status and check for new app versions
    Status(status::Args),
    #[cfg_attr(not(feature = "v2"), no_hook)]
    /// Update Scoop and Scoop buckets
    Update(update::Args),
    /// Opens the app homepage
    Home(home::Args),
}
