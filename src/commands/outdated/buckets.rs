use clap::Parser;
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
        } else {
            for bucket in outdated_buckets {
                println!("❌ `{}` out of date", bucket.name());
            }
        }

        Ok(())
    }
}
