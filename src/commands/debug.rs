use clap::{Parser, Subcommand};
use sfsu_macros::{Hooks, Runnable};
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

mod save;

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
    Save(save::Args),
}

#[derive(Debug, Clone, Parser)]
/// Debugging commands
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()> {
        self.command.run(ctx).await
    }
}
