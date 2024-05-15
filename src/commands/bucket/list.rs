use anyhow::Context;
use chrono::FixedOffset;
use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;
use sprinkles::{buckets::Bucket, config, contexts::ScoopContext, wrappers::time::NicerTime};

use crate::output;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct BucketInfo {
    name: String,
    source: String,
    updated: NicerTime<FixedOffset>,
    manifests: usize,
}

impl BucketInfo {
    fn collect(bucket: &Bucket) -> anyhow::Result<Self> {
        let manifests = bucket.manifests()?;

        let updated_time = {
            let repo = bucket.open_repo()?;
            let latest_commit = repo.latest_commit()?;
            let time = sprinkles::git::parity::Time::from(latest_commit.time()?);

            time.to_datetime().context("invalid time")?
        };

        Ok(Self {
            name: bucket.name().to_string(),
            source: bucket.source()?.to_string(),
            updated: updated_time.into(),
            manifests,
        })
    }
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let buckets = Bucket::list_all(ctx)?;

        let buckets = {
            let mut buckets = buckets
                .par_iter()
                .map(BucketInfo::collect)
                .collect::<Result<Vec<_>, _>>()?;

            buckets.sort_by(|a, b| a.name.cmp(&b.name));

            buckets
        };

        if self.json {
            let output = serde_json::to_string_pretty(&buckets)?;
            println!("{output}");
        } else {
            let structured = output::structured::Structured::new(&buckets);

            println!("{structured}");
        }

        Ok(())
    }
}
