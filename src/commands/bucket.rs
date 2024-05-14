pub mod add;
pub mod known;
pub mod list;
pub mod outdated;
pub mod remove;
pub mod unused;

use clap::{Parser, Subcommand};

use sfsu_derive::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::Command;

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// Add a bucket
    Add(add::Args),
    #[clap(alias = "rm")]
    /// Remove a bucket
    Remove(remove::Args),
    /// List all installed buckets
    List(list::Args),
    /// List all known buckets
    Known(known::Args),
    /// Find buckets that do not have any installed packages
    Unused(unused::Args),
    #[cfg(not(feature = "v2"))]
    /// List outdated buckets
    Outdated(outdated::Args),
}

#[derive(Debug, Clone, Parser)]
/// Commands for managing buckets
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    #[inline]
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        self.command.run(ctx).await
    }
}
