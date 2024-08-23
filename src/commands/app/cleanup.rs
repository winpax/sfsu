use anyhow::Context;
use clap::Parser;
use dialoguer::Confirm;
use futures::future;
use rayon::prelude::*;
use sprinkles::{
    contexts::ScoopContext,
    handles::packages::PackageHandle,
    packages::{reference::package, CreateManifest, Manifest},
    progress::indicatif::{MultiProgress, ProgressBar},
};

use crate::yellow;

#[derive(Debug, Clone, Parser)]
/// Clean up apps by removing old versions
pub struct Args {
    #[clap(short, long, help = "Cleanup all apps")]
    all: bool,

    #[clap(help = "The apps to cleanup")]
    apps: Vec<package::Reference>,

    #[clap(from_global)]
    assume_yes: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let apps = if self.all {
            ctx.installed_apps()?
                .into_par_iter()
                .map(Manifest::from_path)
                .collect::<Result<Vec<_>, _>>()?
        } else {
            future::try_join_all(
                self.apps
                    .into_iter()
                    .map(|reference| async move { reference.manifest(ctx).await }),
            )
            .await?
        };

        if !self.assume_yes
            && !Confirm::new()
                .with_prompt(
                    yellow!(
                        "This will remove old version dirs for {} apps. Are you sure?",
                        apps.len(),
                    )
                    .to_string(),
                )
                .default(false)
                .interact()?
        {
            return Ok(());
        }

        let handles = {
            let future_handles = apps
                .iter()
                .map(|manifest| async move { PackageHandle::from_manifest(ctx, manifest).await });

            future::try_join_all(future_handles).await?
        };

        let mp = MultiProgress::new();

        let pb_style = sprinkles::progress::style(
            Some(sprinkles::progress::ProgressOptions::PosLen),
            Some(sprinkles::progress::Message::suffix()),
        );

        let apps_pb = ProgressBar::new(handles.len() as u64)
            .with_message("Cleaning up")
            .with_style(pb_style.clone());
        mp.add(apps_pb.clone());

        for handle in handles {
            let name = unsafe { handle.name() };

            if handle.running() {
                mp.println(format!("Skipping {name} because it is currently running"))?;
                continue;
            }

            let current_version_dir = handle.version_dir();

            let versions_parent = current_version_dir
                .parent()
                .context("Missing parent directory. This likely means the app is not installed")?;

            let versions = std::fs::read_dir(versions_parent)?
                .filter(|entry| {
                    !matches!(entry, Ok(entry) if entry.path() == current_version_dir
                        || entry.file_name() == "current"
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            let pb = ProgressBar::new(versions.len() as u64)
                .with_message(format!("Cleaning up {name}"))
                .with_style(pb_style.clone());

            mp.add(pb.clone());

            for version_dir in versions {
                let version_dir = version_dir.path();

                std::fs::remove_dir_all(version_dir)?;

                pb.inc(1);
            }

            pb.finish();

            apps_pb.inc(1);
        }

        Ok(())
    }
}
