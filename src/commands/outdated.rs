use clap::{Parser, Subcommand};
use serde_json::Map;
use sfsu_derive::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, DeprecationMessage, DeprecationWarning};

mod apps;
mod buckets;

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// List outdated apps
    Apps(apps::Args),
    /// List outdated buckets
    Buckets(buckets::Args),
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(from_global)]
    json: bool,
}

impl Command for Args {
    fn deprecated() -> Option<DeprecationWarning> {
        Some(DeprecationWarning {
            message: DeprecationMessage::Replacement("sfsu status"),
            version: Some(2.0),
        })
    }

    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        if let Some(command) = self.command {
            command.run(ctx).await
        } else {
            if self.json {
                let mut map = Map::new();

                let apps = apps::Args { json: self.json }
                    .run_direct(ctx, false)?
                    .unwrap_or_default();

                let buckets = buckets::Args { json: self.json }
                    .run_direct(ctx, false)?
                    .unwrap_or_default();

                map.insert("outdated_apps".into(), apps.into());
                map.insert("outdated_buckets".into(), buckets.into());

                let output = serde_json::to_string_pretty(&map)?;

                println!("{output}");
            } else {
                println!("Outdated Apps:");
                Commands::Apps(apps::Args { json: self.json })
                    .run(ctx)
                    .await?;
                println!("\nOutdated Buckets:");
                Commands::Buckets(buckets::Args { json: self.json })
                    .run(ctx)
                    .await?;
            }

            Ok(())
        }
    }
}
