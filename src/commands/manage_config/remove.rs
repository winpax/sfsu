use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

use crate::commands::manage_config::management;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The key to remove")]
    field: String,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, mut ctx: impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let config_manager = management::ManageConfig::new(ctx.config_mut());
        todo!()
    }
}
