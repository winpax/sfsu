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

        let mp = MultiProgress::new();

        let outdated_buckets = buckets
            .into_iter()
            .map(|bucket| {
                let pb = Mutex::new(
                    mp.add(
                        ProgressBar::new(0)
                            .with_message(format!("{}: Pulling updates", bucket.name()))
                            .with_finish(ProgressFinish::WithMessage("Finished pull".into())),
                    ),
                );

                (bucket, pb)
            })
            .filter(|(bucket, pb)| {
                if matches!(bucket.outdated(), Ok(true)) {
                    true
                } else {
                    pb.lock().finish_with_message("Finished pull");
                    false
                }
            })
            .collect::<Vec<_>>();

        outdated_buckets
            .par_iter()
            .try_for_each(|(bucket, pb)| -> buckets::Result<()> {
                let repo = bucket.open_repo()?;

                debug!("Beggining pull for {}", bucket.path().display());

                repo.pull(Some(&|stats, finished| {
                    debug!("Callback for outdated backup pull");
                    let pb = pb.lock();

                    if finished {
                        pb.finish();
                        return true;
                    }

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

                    true
                }))?;

                Ok(())
            })?;

        Ok(())
    }
}
