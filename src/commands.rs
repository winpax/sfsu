pub mod buckets;
pub mod describe;
pub mod hook;
pub mod list;
pub mod search;

use clap::Subcommand;

use sfsu_derive::{Hooks, Runnable};

pub trait Command {
    fn run(self) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Hooks, Clone, Subcommand, Runnable)]
pub enum Commands {
    Buckets(buckets::Args),
    Search(search::Args),
    List(list::Args),
    Hook(hook::Args),
    Describe(describe::Args),
}
