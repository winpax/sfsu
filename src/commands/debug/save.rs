use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
/// Save the current config
pub struct Args;

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        ctx.config().save()?;

        Ok(())
    }
}
