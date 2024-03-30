pub mod unused;

use clap::{Parser, Subcommand};

use sfsu_derive::{Hooks, Runnable};

use super::Command;

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// Find buckets that do not have any installed packages
    Unused(unused::Args),
}

#[derive(Debug, Clone, Parser)]
/// Commands for managing buckets
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    #[inline]
    fn runner(self) -> Result<(), anyhow::Error> {
        self.command.run()
    }
}
