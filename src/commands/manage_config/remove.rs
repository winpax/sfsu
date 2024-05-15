use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The key to remove")]
    field: String,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        todo!()
    }
}
