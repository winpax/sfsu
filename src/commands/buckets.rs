pub mod unused;

use clap::{Parser, Subcommand};

use sfsu_derive::{Hooks, Runnable};

pub trait Command {
    fn run(self) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
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
    fn run(self) -> Result<(), anyhow::Error> {
        self.command.run()
    }
}
