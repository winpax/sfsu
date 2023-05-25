pub mod describe;
pub mod hook;
pub mod list;
pub mod search;
pub mod unused;

use clap::Subcommand;

// TODO: Better way of doing this? or add support for meta in proc macro
use sfsu_derive::RawEnum;

pub trait Command {
    type Error;

    fn run(self) -> Result<(), Self::Error>;
}

#[derive(Debug, RawEnum, Clone, Subcommand)]
pub enum Commands {
    Search(search::Args),
    List(list::Args),
    Hook(hook::Args),
    UnusedBuckets(unused::Args),
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
