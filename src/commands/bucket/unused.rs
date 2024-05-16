use clap::Parser;

use rayon::prelude::*;
use sprinkles::{buckets::Bucket, config, contexts::ScoopContext, packages::InstallManifest};

use crate::{
    commands,
    output::sectioned::{Children, Section},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl commands::Command for Args {
    async fn runner(self, ctx: impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        // TODO: Refactor
        let used_buckets = InstallManifest::list_all_unchecked(&ctx)?
            .par_iter()
            .filter_map(|entry| entry.bucket.clone())
            .collect::<Vec<_>>();

        let unused_buckets = Bucket::list_all(&ctx)?
            .par_iter()
            .filter_map(|bucket| {
                if used_buckets.contains(&bucket.name().to_string()) {
                    None
                } else {
                    Some((bucket.name()).to_string())
                }
            })
            .collect::<Vec<_>>();

        if self.json {
            let output = serde_json::to_string_pretty(&unused_buckets)?;
            println!("{output}");
        } else {
            let unused_buckets = Children::from(unused_buckets);
            if let Children::None = unused_buckets {
                println!("No unused buckets");
            } else {
                let unused =
                    Section::new(unused_buckets).with_title("The following buckets are unused:");
                println!("{unused}");
            };
        }

        Ok(())
    }
}
