use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressFinish};
use itertools::Itertools;
use rayon::prelude::*;

use sprinkles::{
    buckets::{self, Bucket},
    config::Scoop as ScoopConfig,
    git::__stats_callback,
    output::sectioned::{Children, Section},
    progress::{style, Message, ProgressOptions},
    Scoop,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Show commit messages for each update")]
    changelog: bool,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        const FINISH_MESSAGE: &str = "✅";

        let progress_style = style(Some(ProgressOptions::Hide), Some(Message::Suffix(None)));

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
            .with_prefix(format!("🍨 {:<longest_bucket_name$}", "Scoop"))
            .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into()));

        let scoop_changelog = if scoop_repo.outdated()? {
            let mut changelog = if self.changelog {
                scoop_repo.pull_with_changelog(Some(&|stats, thin| {
                    __stats_callback(&stats, thin, &pb);
                    true
                }))?
            } else {
                scoop_repo.pull(Some(&|stats, thin| {
                    __stats_callback(&stats, thin, &pb);
                    true
                }))?;
                vec![]
            };

            pb.finish_with_message(FINISH_MESSAGE);

            changelog.reverse();

            Some(changelog)
        } else {
            pb.finish_with_message("✅ No updates available");

            None
        };

        let mp = MultiProgress::new();

        let outdated_buckets = buckets
            .into_iter()
            .map(|bucket| {
                let pb = mp.add(
                    ProgressBar::new(1)
                        .with_style(progress_style.clone())
                        .with_message("Checking updates")
                        .with_prefix(format!("🪣 {:<longest_bucket_name$}", bucket.name()))
                        .with_finish(ProgressFinish::WithMessage(FINISH_MESSAGE.into())),
                );

                pb.set_position(0);

                (bucket, pb)
            })
            .collect_vec();

        let bucket_changelogs = outdated_buckets
            .par_iter()
            .map(|(bucket, pb)| -> buckets::Result<(String, Vec<String>)> {
                let repo = bucket.open_repo()?;

                if !repo.outdated()? {
                    pb.finish_with_message("✅ No updates available");
                    return Ok((bucket.name().to_string(), vec![]));
                }

                debug!("Beggining pull for {}", bucket.name());

                let changelog = if self.changelog {
                    repo.pull_with_changelog(Some(&|stats, thin| {
                        __stats_callback(&stats, thin, pb);
                        true
                    }))?
                } else {
                    repo.pull(Some(&|stats, thin| {
                        __stats_callback(&stats, thin, pb);
                        true
                    }))?;

                    vec![]
                };

                pb.finish_with_message(FINISH_MESSAGE);

                Ok((bucket.name().to_string(), changelog))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut scoop_config = ScoopConfig::load()?;
        scoop_config.update_last_update_time();
        scoop_config.save()?;

        if self.changelog {
            println!();
            if let Some(scoop_changelog) = scoop_changelog {
                let scoop_changelog =
                    Section::new(Children::from(scoop_changelog)).with_title("Scoop changes:");

                print!("{scoop_changelog}");
            };

            for bucket_changelog in bucket_changelogs {
                let (name, changelog) = bucket_changelog;

                if changelog.is_empty() {
                    continue;
                }

                let changelog =
                    Section::new(Children::from(changelog)).with_title(format!("{name} changes:"));

                println!("{changelog}");
            }
        }

        Ok(())
    }
}
