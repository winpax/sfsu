use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The key to set")]
    field: String,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, mut ctx: impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let config_manager = super::management::ManageConfig::new(ctx.config_mut());

        let value = config_manager.get(self.field)?;

        if self.json {
            println!("{}", serde_json::to_string_pretty(&value)?);
        } else {
            let value = if let Some(str) = value.as_str() {
                str.to_owned()
            } else {
                value.to_string()
            };

            println!("{value}");
        }

        Ok(())
    }
}
