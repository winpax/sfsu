use clap::Parser;

use crate::commands;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl commands::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
