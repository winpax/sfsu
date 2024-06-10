pub mod purge;

use clap::{Parser, Subcommand};

use sfsu_macros::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// Purge package's persist folder
    Purge(purge::Args),
}

#[derive(Debug, Clone, Parser)]
/// Commands for managing apps
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl Command for Args {
    #[inline]
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        self.command.run(ctx).await
    }
}
