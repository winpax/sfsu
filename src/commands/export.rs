use clap::Parser;

use rayon::prelude::*;
use sfsu::{
    buckets::Bucket,
    config,
    summary::{bucket, package, Summaries},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Export the scoop config too")]
    config: bool,
}

impl super::Command for Args {
    fn run(self) -> anyhow::Result<()> {
        let buckets = Bucket::list_all()?
            .into_iter()
            .map(bucket::Summary::from_bucket)
            .collect::<Result<Vec<_>, _>>()?;

        let packages = sfsu::Scoop::list_installed_scoop_apps()?
            .into_par_iter()
            .map(package::Summary::from_path)
            .collect::<Result<Vec<_>, _>>()?;

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
