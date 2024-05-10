pub mod add;
pub mod outdated;
pub mod unused;

use clap::{Parser, Subcommand};

use sfsu_derive::{Hooks, Runnable};
use sprinkles::{config, contexts::ScoopContext};

use super::Command;

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// Add a bucket
    Add(add::Args),
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
