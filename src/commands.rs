pub mod describe;
pub mod hook;
pub mod list;
pub mod search;
pub mod unused;

use clap::Subcommand;
use sfsu_derive::RawEnum;

pub trait Command {
    type Error;

    fn run(self) -> Result<(), Self::Error>;
}

#[derive(Debug, RawEnum, Clone, Subcommand)]
pub enum Commands {
    #[command(about = "Search for a package")]
    Search(search::Args),
    #[command(about = "List all installed packages")]
    List(list::Args),
    #[command(about = "Generate PowerShell hook")]
    Hook(hook::Args),
    #[command(about = "Find buckets that do not have any installed packages")]
    UnusedBuckets(unused::Args),
    #[command(about = "Describe a package")]
    Describe(describe::Args),
}

impl Commands {
    pub fn run(self) -> Result<(), anyhow::Error> {
        // TODO: Find a way to unpack inner value without match statement
        match self {
            Commands::Search(args) => args.run()?,
            Commands::List(args) => args.run()?,
            Commands::Hook(args) => args.run()?,
            Commands::UnusedBuckets(args) => args.run()?,
            Commands::Describe(args) => args.run()?,
        }

        Ok(())
    }
}
