use clap::Parser;
use rayon::prelude::*;

use sfsu::{
    buckets::{self, Bucket},
    git,
};

use crate::commands;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl commands::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let buckets = Bucket::list_all()?;

        let outdated_buckets = buckets
            .into_iter()
            .filter(|bucket| matches!(bucket.outdated(), Ok(true)))
            .collect::<Vec<_>>();

        outdated_buckets
            .par_iter()
            .try_for_each(|bucket| -> buckets::Result<()> {
                let repo = bucket.open_repo()?;

                Ok(())
            });

        todo!()
    }
}
