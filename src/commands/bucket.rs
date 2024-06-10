pub mod add;
pub mod known;
pub mod list;
pub mod outdated;
pub mod remove;
pub mod unused;

use clap::{Parser, Subcommand};

use sfsu_macros::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    Add(add::Args),
    #[clap(alias = "rm")]
    Remove(remove::Args),
    List(list::Args),
    Known(known::Args),
    Unused(unused::Args),
    #[cfg(not(feature = "v2"))]
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
