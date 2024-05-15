use clap::Parser;

use sprinkles::{config, contexts::ScoopContext};

use crate::models::export::Export;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Export the scoop config as well")]
    config: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let export = {
            let mut export = Export::load(ctx)?;

            if !self.config {
                export.config = None;
            }

            export
        };

        let output = serde_json::to_string_pretty(&export)?;

        println!("{output}");

        Ok(())
    }
}
