use clap::{Parser, Subcommand};
use sfsu_derive::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

pub mod export;

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// Exports installed apps, buckets (and optionally configs) in JSON format
    Export(export::Args),
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Commands,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        self.command.run(ctx).await
    }
}
