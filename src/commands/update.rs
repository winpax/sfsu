use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish};
use itertools::Itertools;
use rayon::prelude::*;

use sfsu::{
    buckets::{self, Bucket},
    config::Scoop as ScoopConfig,
    progress::{style, MessagePosition, ProgressOptions},
    Scoop,
};

#[derive(Debug, Clone, Parser)]
pub struct Args;

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        const FINISH_MESSAGE: &str = "✅";

        let progress_style = style(Some(ProgressOptions::Hide), Some(MessagePosition::Suffix));

        let buckets = Bucket::list_all()?;

        let longest_bucket_name = buckets
            .iter()
            .map(|bucket| bucket.name().len())
            .max()
            .unwrap_or(0);

        let scoop_repo = Scoop::open_repo()?;

        let pb = ProgressBar::new(1)
            .with_style(progress_style.clone())
            .with_message("Checking for updates")
            .with_prefix(format!("{:<longest_bucket_name$}", "Scoop"))
            .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into()));

        if scoop_repo.outdated()? {
            scoop_repo.pull(Some(&|stats, thin| {
                if thin {
                    pb.set_position(stats.indexed_objects() as u64);
                    pb.set_length(stats.total_objects() as u64);

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

                true
            }))?;

            pb.finish_with_message(FINISH_MESSAGE);
        } else {
            pb.finish_with_message("✅ No updates available");
        }

        let mp = MultiProgress::new();

        let outdated_buckets = buckets
            .into_iter()
            .map(|bucket| {
                let pb = mp.add(
                    ProgressBar::new(1)
                        .with_style(progress_style.clone())
                        .with_message("Checking updates")
                        .with_prefix(format!("{:<longest_bucket_name$}", bucket.name()))
                        .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into())),
                );

                pb.set_position(0);

                (bucket, pb)
            })
            .collect_vec();

        outdated_buckets
            .par_iter()
            .try_for_each(|(bucket, pb)| -> buckets::Result<()> {
                let repo = bucket.open_repo()?;

                if !repo.outdated()? {
                    pb.finish_with_message("✅ No updates available");
                    return Ok(());
                }

                debug!("Beggining pull for {}", bucket.name());

                repo.pull(Some(&|stats, thin| {
                    debug!("Callback for outdated backup pull");

                    if thin {
                        pb.set_position(stats.indexed_objects() as u64);
                        pb.set_length(stats.total_objects() as u64);

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

                    true
                }))?;

                Ok(())
            })?;

        let mut scoop_config = ScoopConfig::load()?;
        scoop_config.update_last_update_time();
        scoop_config.save()?;

        Ok(())
    }
}
