use anyhow::Context;
use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

use crate::{abandon, calm_panic::CalmUnwrap};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The name of the bucket to add")]
    name: String,

    #[clap(help = "The url of the bucket to add")]
    repo: Option<String>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let _repo_url = self
            .repo
            .clone()
            .context("No repo provided")
            .unwrap_or_else(|_| {
                let known_buckets = ctx
                    .known_buckets()
                    .calm_expect("Failed to decode known buckets");

                if let Some(url) = known_buckets.get(&self.name) {
                    url.to_string()
                } else {
                    abandon!(
                        "No bucket found with the name \"{}\". Try passing the url as well",
                        self.name
                    )
                }
            });

        todo!()
    }
}
