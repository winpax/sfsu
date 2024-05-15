use clap::Parser;
use itertools::Itertools;
use sprinkles::{buckets::Bucket, config, contexts::ScoopContext};

use crate::{
    commands::{self, DeprecationMessage, DeprecationWarning},
    output::sectioned::{Children, Section},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl commands::Command for Args {
    const DEPRECATED: Option<DeprecationWarning> = Some(DeprecationWarning {
        message: DeprecationMessage::Replacement("sfsu status"),
        version: Some(2.0),
    });

    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let buckets = Bucket::list_all(ctx)?;

        let outdated_buckets = buckets
            .into_iter()
            .filter_map(|bucket| {
                bucket.outdated().ok().and_then(|outdated| {
                    if outdated {
                        Some(bucket.name().to_string())
                    } else {
                        None
                    }
                })
            })
            .collect_vec();

        if outdated_buckets.is_empty() {
            eprintln!("All buckets up to date.");
        } else if self.json {
            let json = serde_json::to_string_pretty(&outdated_buckets)?;

            println!("{json}");
        } else {
            let title = format!("{} outdated buckets:", outdated_buckets.len());

            let section = Section::new(Children::from(outdated_buckets)).with_title(title);

            println!("{section}");
        }

        Ok(())
    }
}
