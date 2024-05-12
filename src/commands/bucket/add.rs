use once_cell::sync::Lazy;
use tokio::process::Command;

use anyhow::Context;
use clap::Parser;
use sprinkles::{
    config,
    contexts::ScoopContext,
    progress::{
        indicatif::{MultiProgress, ProgressBar},
        style, ProgressOptions,
    },
};

use crate::{abandon, calm_panic::CalmUnwrap};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The name of the bucket to add")]
    name: String,

    #[clap(help = "The url of the bucket to add")]
    repo: Option<String>,

    #[clap(from_global)]
    disable_git: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let repo_url = self
            .repo
            .clone()
            .context("No repo provided")
            .unwrap_or_else(|_| {
                let known_buckets = ctx
                    .known_buckets()
                    .calm_expect("Failed to decode known buckets");

                if let Some(url) = known_buckets.get(&self.name) {
                    url.to_string()
                } else {
                    abandon!(
                        "No bucket found with the name \"{}\". Try passing the url as well",
                        self.name
                    )
                }
            });

        let dest_path = ctx.buckets_path().join(&self.name);

        if dest_path.exists() {
            abandon!("Bucket {name} already exists. Remove it first if you want to add it again: `sfsu bucket rm {name}`", name = self.name);
        }

        println!("Cloning {} into {}", repo_url, dest_path.display());

        if self.disable_git {
            let default_style = style(Some(ProgressOptions::PosLen), None);
            let pb = || ProgressBar::new(0).with_style(default_style.clone());

            let fetch_progress = MultiProgress::new();
            let recv_progress = Lazy::new(|| {
                let bar = fetch_progress.add(pb());
                bar.set_message("Receiving objects");
                bar
            });

            let index_progress = Lazy::new(|| {
                let bar = fetch_progress.add(pb());
                bar.set_message("Indexing objects");
                bar
            });

            let index_deltas = Lazy::new(|| {
                let bar = fetch_progress.add(pb());
                bar.set_message("Indexing deltas");
                bar
            });
            let checkout_progress = Lazy::new(|| {
                let bar = fetch_progress.add(pb());
                bar.set_message("Checking out");
                bar
            });

            sprinkles::git::clone::clone(
                &repo_url,
                dest_path,
                |stats| {
                    recv_progress.set_position(stats.received_objects() as u64);
                    recv_progress.set_length(stats.total_objects() as u64);

                    index_progress.set_position(stats.indexed_objects() as u64);
                    index_progress.set_length(stats.total_objects() as u64);

                    if stats.indexed_deltas() > 0 {
                        index_deltas.set_position(stats.indexed_deltas() as u64);
                        index_deltas.set_length(stats.total_deltas() as u64);
                    }
                },
                |path, curr, total| {
                    checkout_progress.set_position(curr as u64);
                    checkout_progress.set_length(total as u64);
                    if let Some(path) = path {
                        checkout_progress.set_message(format!("Checking out {}", path.display()));
                    }
                },
            )?;
        } else {
            let git_path = sprinkles::git::which().calm_expect("git not found");

            let exit_status = Command::new(git_path)
                .current_dir(ctx.buckets_path())
                .arg("clone")
                .arg(repo_url)
                .arg(self.name)
                .spawn()?
                .wait_with_output()
                .await?;

            match exit_status.status.code() {
                Some(0) => {}
                Some(code) => {
                    return Err(anyhow::anyhow!(
                        "git exited with code {}.\nOutput:\n{}",
                        code,
                        String::from_utf8_lossy(&exit_status.stdout)
                    ))
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "git exited without a status code.\nOutput:\n{}",
                        String::from_utf8_lossy(&exit_status.stdout)
                    ))
                }
            }
        };

        Ok(())
    }
}
