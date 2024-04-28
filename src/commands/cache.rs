use clap::{Parser, Subcommand};
use sfsu_derive::Runnable;

mod rm;
mod show;

use crate::commands::Command;

#[derive(Debug, Clone, Subcommand, Runnable)]
enum Commands {
    /// List cache entries
    Show(show::Args),
    /// Remove cache entries
    Rm(rm::Args),
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let command = self
            .command
            .unwrap_or(Commands::Show(show::Args { json: self.json }));

        command.run().await
    }
}
