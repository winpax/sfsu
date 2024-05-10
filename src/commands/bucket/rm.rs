use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The name of the bucket to delete")]
    name: String,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        Ok(())
    }
}
