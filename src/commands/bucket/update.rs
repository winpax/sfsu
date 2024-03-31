use std::sync::atomic::{AtomicBool, Ordering};

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish};
use itertools::Itertools;
use parking_lot::Mutex;
use rayon::prelude::*;

use sfsu::{
    buckets::{self, Bucket},
    progress::{style, MessagePosition, ProgressOptions},
};

use crate::commands;

#[derive(Debug, Clone, Parser)]
pub struct Args;

impl commands::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        // TODO: Add update time to bucket so scoop doesn't do it again

        const FINISH_MESSAGE: &str = "✅";

        let buckets = Bucket::list_all()?;

        let progress_style = style(None, Some(MessagePosition::Suffix));

        let mp = MultiProgress::new();

        let longest_bucket_name = buckets
            .iter()
            .map(|bucket| bucket.name().len())
            .max()
            .unwrap_or(0);

        let outdated_buckets = buckets
            .into_iter()
            .map(|bucket| {
                let pb = Mutex::new(
                    mp.add(
                        ProgressBar::new(1)
                            .with_position(0)
                            .with_style(progress_style.clone())
                            .with_message("Checking bucket for updates")
                            .with_prefix(format!("{:<longest_bucket_name$}", bucket.name()))
                            .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into())),
                    ),
                );

                pb.lock().set_position(0);

                (bucket, pb)
            })
            .collect_vec();

        outdated_buckets
            .par_iter()
            .try_for_each(|(bucket, pb)| -> buckets::Result<()> {
                let repo = bucket.open_repo()?;

                debug!("Beggining pull for {}", bucket.path().display());

                let immediate = AtomicBool::new(true);

                repo.pull(Some(&|stats, finished| {
                    debug!("Callback for outdated backup pull");
                    let pb = pb.lock();

                    if finished {
                        // Thin pack
                        pb.set_position(stats.indexed_objects() as u64);
                        pb.set_length(stats.total_objects() as u64);

                        if immediate.load(Ordering::Relaxed) {
                            pb.set_style(style(
                                Some(ProgressOptions::Hide),
                                Some(MessagePosition::Suffix),
                            ));
                        }

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
                    }

                    immediate.store(false, Ordering::Relaxed);

                    true
                }))?;

                Ok(())
            })?;

        Ok(())
    }
}
