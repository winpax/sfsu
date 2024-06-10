use clap::Parser;
use itertools::Itertools;
use serde::Serialize;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Serialize)]
struct KnownBucket {
    name: String,
    source: String,
}

#[derive(Debug, Clone, Parser)]
/// List all known buckets
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let known_buckets = ctx.known_buckets();
        let known_buckets = known_buckets
            .into_iter()
            .map(|(name, source)| {
                let name = (*name).to_string();
                let source = (*source).to_string();
                KnownBucket { name, source }
            })
            .collect_vec();

        if self.json {
            let output = serde_json::to_string_pretty(&known_buckets)?;
            println!("{output}");
        } else {
            let structured = crate::output::structured::Structured::new(&known_buckets);

            println!("{structured}");
        }

        Ok(())
    }
}
