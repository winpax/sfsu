use clap::Parser;
use dialoguer::Confirm;
use sprinkles::contexts::ScoopContext;

use crate::{abandon, output::colours::yellow};

#[derive(Debug, Clone, Parser)]
/// Remove a bucket
pub struct Args {
    #[clap(help = "The name of the bucket to delete")]
    name: String,

    #[clap(from_global)]
    assume_yes: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let path = ctx.buckets_path().join(&self.name);

        if !path.exists() {
            abandon!("Bucket \"{}\" is not installed", self.name);
        }

        if self.name == "main" && !self.assume_yes {
            Confirm::new()
            .with_prompt(yellow!("You probably don't want to delete the main bucket. Are you sure you want to continue?").to_string())
            .default(false)
            .interact()?;
        }

        let response = if self.assume_yes {
            true
        } else {
            Confirm::new()
                .with_prompt(format!(
                    "Are you sure you want to delete \"{}\"?",
                    path.display()
                ))
                .default(false)
                .interact()?
        };

        if response {
            tokio::fs::remove_dir_all(path).await?;
        }

        Ok(())
    }
}
