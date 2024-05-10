use std::process::Command;

use anyhow::Context;
use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

use crate::{abandon, calm_panic::CalmUnwrap};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The name of the bucket to add")]
    name: String,

    #[clap(help = "The url of the bucket to add")]
    repo: Option<String>,

    #[clap(from_global)]
    disable_git: bool,

    #[clap(from_global)]
    json: bool,
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

        if self.disable_git {
            abandon!("Disabling git for buckets is not yet supported");
        } else {
            let git_path = sprinkles::git::which().calm_expect("git not found");

            let exit_status = Command::new(git_path)
                .arg("clone")
                .arg(repo_url)
                .arg(self.name)
                .spawn()?
                .wait_with_output()?;

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
