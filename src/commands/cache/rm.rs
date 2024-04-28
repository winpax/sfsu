use clap::Parser;

use crate::commands::Command;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    apps: Vec<String>,

    #[clap(from_global)]
    json: bool,
}

impl Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
