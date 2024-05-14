use anyhow::Context;
use clap::Parser;
use itertools::Itertools;
use rayon::prelude::*;

use sprinkles::{
    buckets::{self, Bucket},
    config::{self, Scoop as ScoopConfig},
    contexts::ScoopContext,
    git::{self, Repo, __stats_callback},
    progress::{
        indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle},
        style, Message, ProgressOptions,
    },
};

use crate::output::sectioned::{Children, Section};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Show commit messages for each update")]
    changelog: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        let progress_style = style(Some(ProgressOptions::Hide), Some(Message::suffix()));

        let buckets = Bucket::list_all(ctx)?;

        let longest_bucket_name = buckets
            .iter()
            .map(|bucket| bucket.name().len())
            .max()
            .unwrap_or(0);

        // Force checkout to the config's branch
        _ = ctx.outdated().await?;

        let scoop_changelog =
            self.update_scoop(ctx, longest_bucket_name, progress_style.clone())?;

        let mp = MultiProgress::new();

        let outdated_buckets = buckets
            .into_iter()
            .map(|bucket| {
                let pb = mp.add(
                    ProgressBar::new(1)
                        .with_style(progress_style.clone())
                        .with_message("Checking updates")
                        .with_prefix(format!("🪣 {:<longest_bucket_name$}", bucket.name()))
                        .with_finish(ProgressFinish::WithMessage(Self::FINISH_MESSAGE.into())),
                );

                pb.set_position(0);

                (bucket, pb)
            })
            .collect_vec();

        let bucket_changelogs = self.update_buckets(ctx, &outdated_buckets)?;

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

impl Args {
    const FINISH_MESSAGE: &'static str = "✅";

    fn update_scoop(
        &self,
        ctx: &impl ScoopContext<config::Scoop>,
        longest_bucket_name: usize,
        style: ProgressStyle,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let repo = ctx.open_repo().context("missing user repository")??;

        let pb = ProgressBar::new(1)
            .with_style(style)
            .with_message("Checking for updates")
            .with_prefix(format!("🍨 {:<longest_bucket_name$}", "Scoop"))
            .with_finish(ProgressFinish::WithMessage(Self::FINISH_MESSAGE.into()));

        let changelog = self.update(ctx, &repo, &pb)?;

        Ok(changelog)
    }

    fn update_buckets(
        &self,
        ctx: &impl ScoopContext<config::Scoop>,
        outdated_buckets: &[(Bucket, ProgressBar)],
    ) -> anyhow::Result<Vec<(String, Vec<String>)>> {
        let bucket_changelogs = outdated_buckets
            .par_iter()
            .map(|(bucket, pb)| -> buckets::Result<(String, Vec<String>)> {
                let repo = bucket.open_repo()?;

                let changelog = self.update(ctx, &repo, pb)?;

                Ok((bucket.name().to_string(), changelog.unwrap_or_default()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(bucket_changelogs)
    }

    fn update(
        &self,
        ctx: &impl ScoopContext<config::Scoop>,
        repo: &Repo,
        pb: &ProgressBar,
    ) -> git::Result<Option<Vec<String>>> {
        if !repo.outdated()? {
            pb.finish_with_message("✅ No updates available");
            return Ok(None);
        }

        let changelog = if self.changelog {
            repo.pull_with_changelog(
                ctx,
                Some(&|stats, thin| {
                    __stats_callback(&stats, thin, pb);
                    true
                }),
            )?
        } else {
            repo.pull(
                ctx,
                Some(&|stats, thin| {
                    __stats_callback(&stats, thin, pb);
                    true
                }),
            )?;

            vec![]
        };

        pb.finish_with_message(Self::FINISH_MESSAGE);

        Ok(Some(changelog))
    }
}
