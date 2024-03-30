use std::mem::MaybeUninit;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish};
use parking_lot::Mutex;
use rayon::prelude::*;

use sfsu::buckets::{self, Bucket};

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

        let mp = MultiProgress::new();

        outdated_buckets
            .par_iter()
            .try_for_each(|bucket| -> buckets::Result<()> {
                let repo = bucket.open_repo()?;

                let pb = Mutex::new(None::<ProgressBar>);

                println!("starting doing shit lets go yay");

                repo.pull(Some(&mut |stats| {
                    if let Some(pb) = pb.lock().as_ref() {
                        if stats.received_objects() == stats.total_objects() {
                            pb.set_position(stats.indexed_deltas() as u64);
                            pb.set_length(stats.total_deltas() as u64);
                            pb.set_message("Resolving deltas");
                        } else if stats.total_objects() > 0 {
                            pb.set_position(stats.received_objects() as u64);
                            pb.set_length(stats.total_objects() as u64);
                            pb.set_message("Receiving objects");
                            // eprint!(
                            //     "Received {}/{} objects ({}) in {} bytes\r",
                            //     stats.received_objects(),
                            //     stats.total_objects(),
                            //     stats.indexed_objects(),
                            //     stats.received_bytes()
                            // );
                        }
                    } else {
                        *pb.lock() = Some(
                            mp.add(ProgressBar::new(stats.total_deltas() as u64))
                                .with_message("Resolving deltas")
                                .with_finish(ProgressFinish::WithMessage("Finished pull".into())),
                        );

                        println!("Fetching from {}", bucket.path().display());
                    }

                    true
                }))?;

                Ok(())
            })?;

        Ok(())
    }
}
