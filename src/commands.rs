pub mod describe;
pub mod hook;
pub mod list;
pub mod search;
pub mod unused;

use clap::Subcommand;

// TODO: Better way of doing this? or add support for meta in proc macro
use sfsu_derive::{RawEnum, Runnable};

pub trait Command {
    fn run(self) -> Result<(), anyhow::Error>;
}

#[derive(Debug, RawEnum, Clone, Subcommand, Runnable)]
pub enum Commands {
    Search(search::Args),
    List(list::Args),
    Hook(hook::Args),
    UnusedBuckets(unused::Args),
    Describe(describe::Args),
}
