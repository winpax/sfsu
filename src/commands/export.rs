use clap::Parser;

use rayon::prelude::*;
use sfsu::{
    buckets::Bucket,
    config,
    summary::{bucket, package, Summaries},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Export the scoop config as well")]
    config: bool,
}

impl super::Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        let buckets = Bucket::list_all()?
            .par_iter()
            .map(bucket::Summary::from_bucket)
            .collect::<Result<Vec<_>, _>>()?;

        let packages = package::Summary::parse_all()?;

        let summaries = Summaries {
            buckets,
            packages,
            config: self.config.then(config::Scoop::load).transpose()?,
        };

        let output = serde_json::to_string_pretty(&summaries)?;

        println!("{output}");

        Ok(())
    }
}
