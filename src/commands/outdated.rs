use clap::{Parser, Subcommand};
use sfsu_derive::Runnable;

use super::Command;

mod apps;
mod buckets;

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// List outdated apps
    Apps(apps::Args),
    /// List outdated buckets
    Buckets(buckets::Args),
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(from_global)]
    json: bool,
}

impl Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        if let Some(command) = self.command {
            command.run()
        } else {
            println!("Outdated Apps:");
            Commands::Apps(apps::Args { json: self.json }).run()?;
            println!("Outdated Buckets:");
            Commands::Buckets(buckets::Args { json: self.json }).run()?;

            Ok(())
        }
    }
}
