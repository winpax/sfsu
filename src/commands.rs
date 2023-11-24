pub mod describe;
pub mod hook;
pub mod list;
pub mod search;
pub mod unused;

use clap::Subcommand;

use sfsu_derive::{Hooks, Runnable};

pub trait Command {
    fn run(self) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
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
}
