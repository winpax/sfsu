use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;
use sfsu::buckets::Bucket;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    pub(super) json: bool,
}

impl super::super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let outdated_buckets = Bucket::list_all()?
            .into_par_iter()
            .filter(|bucket| match bucket.outdated() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Bucket Error: {e}");
                    false
                }
            })
            .collect::<Vec<_>>();

        if outdated_buckets.is_empty() {
            println!("All buckets up to date!");
        } else if self.json {
            let outdated_bucket_names = outdated_buckets
                .into_iter()
                .map(|bucket| bucket.name().to_string())
                .collect_vec();

            let output = serde_json::to_string_pretty(&outdated_bucket_names)?;

            println!("{output}");
        } else {
            for bucket in outdated_buckets {
                println!("❌ `{}` bucket is out of date", bucket.name());
            }
        }

        Ok(())
    }
}
