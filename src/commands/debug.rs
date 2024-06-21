use clap::{Parser, Subcommand};
use sfsu_macros::{Hooks, Runnable};
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

mod save;
pub mod sizes;

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
    Save(save::Args),
    /// Show the size of each of the sfsu commands
    Sizes(sizes::Args),
}

#[derive(Debug, Clone, Parser)]
/// Debugging commands
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        self.command.run(ctx).await
    }
}
