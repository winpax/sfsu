pub mod buckets;
pub mod describe;
pub mod hook;
pub mod info;
pub mod list;
pub mod outdated;
pub mod search;

use clap::Subcommand;

use sfsu_derive::{Hooks, Runnable};

// TODO: Run command could return `impl Display` and print that itself
pub trait Command {
    fn deprecated() -> Option<&'static str> {
        None
    }

    fn runner(self) -> Result<(), anyhow::Error>;

    fn run(self) -> Result<(), anyhow::Error>
    where
        Self: Sized,
    {
        if let Some(deprecation_message) = Self::deprecated() {
            use colored::Colorize as _;

            let mut output = String::from("DEPRECATED: ");
            output += deprecation_message;

            println!("{}\n", output.yellow());
        }

        self.runner()
    }
}

#[derive(Debug, Clone, Subcommand, Hooks, Runnable)]
pub enum Commands {
    /// Commands to manage scoop buckets
    Buckets(buckets::Args),
    /// Search for a package
    Search(search::Args),
    /// List all installed packages
    List(list::Args),
    /// Generate hooks for the given shell
    Hook(hook::Args),
    /// Find buckets that do not have any installed packages
    UnusedBuckets(unused::Args),
    /// Describe a package
    Describe(describe::Args),
    /// Display information about a package
    Info(info::Args),
    /// List outdated packages
    Outdated(outdated::Args),
}
