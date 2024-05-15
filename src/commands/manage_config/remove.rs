use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The key to remove")]
    field: String,
}

impl super::Command for Args {
    async fn runner(self, mut ctx: impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let mut config_manager = super::management::ManageConfig::new(ctx.config_mut());

        config_manager.remove(&self.field)?;

        println!("'{}' removed from config", self.field);

        Ok(())
    }
}
