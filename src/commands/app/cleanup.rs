use std::{io::Write, path::Path};

use anyhow::Context;
use clap::Parser;
use dialoguer::Confirm;
use futures::future;
use rayon::prelude::*;
use sprinkles::{
    contexts::ScoopContext,
    handles::packages::PackageHandle,
    packages::{
        reference::{manifest, package},
        CreateManifest, Manifest,
    },
    progress::indicatif::{MultiProgress, ProgressBar},
};

use crate::{
    files::sizes::get_recursive_size, logging::threaded::THREADED_LOGGER, output::colours::yellow,
};

fn handle_path(path: &Path) -> Option<Manifest> {
    if let Ok(manifest) = Manifest::from_path(path.join("current/manifest.json")) {
        Some(manifest.with_name(path.file_name().unwrap().to_string_lossy().to_string()))
    } else {
        writeln!(
            THREADED_LOGGER.lock(),
            "{}",
            yellow!("App installed at {} was invalid.", path.display())
        )
        .ok()?;
        None
    }
}

#[derive(Debug, Clone, Parser)]
/// Clean up apps by removing old versions
pub struct Args {
    #[clap(short, long, help = "Cleanup all apps")]
    all: bool,

    #[clap(help = "The apps to cleanup")]
    apps: Vec<package::Reference>,

    #[clap(from_global)]
    assume_yes: bool,

    // TODO: make this global and add support for more commands
    #[clap(long, help = "Dry run. Do not modify anything on disk")]
    dry_run: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let apps = if self.all {
            ctx.installed_apps()?
                .par_iter()
                .filter_map(|path| handle_path(path))
                .collect::<Vec<_>>()
        } else {
            future::try_join_all(
                self.apps
                    .iter()
                    .map(|reference| async move { reference.manifest(ctx).await }),
            )
            .await?
        };

        let handles = {
            let future_handles = apps.iter().map(|manifest| async move {
                // TODO: Remove before release
                {
                    let reference = manifest::Reference::BucketNamePair {
                        bucket: manifest.bucket_opt().unwrap().to_string(),
                        name: manifest.name_opt().unwrap().to_string(),
                    };

                    let pakref: package::Reference = reference.into();

                    dbg!(pakref.name());
                }

                PackageHandle::from_manifest(ctx, manifest).await
            });

            future::join_all(future_handles)
                .await
                .into_iter()
                .filter_map(std::result::Result::ok)
                .collect::<Vec<_>>()
        };

        if !self.assume_yes
            && !Confirm::new()
                .with_prompt(
                    yellow!(
                        "This will remove old version dirs for {} apps. Are you sure?",
                        handles.len(),
                    )
                    .to_string(),
                )
                .default(false)
                .interact()?
        {
            return Ok(());
        }

        self.handle_apps(&handles)?;

        Ok(())
    }
}

impl Args {
    fn handle_apps(&self, handles: &[PackageHandle<'_, impl ScoopContext>]) -> anyhow::Result<()> {
        let mp = MultiProgress::new();

        let pb_style = sprinkles::progress::style(
            Some(sprinkles::progress::ProgressOptions::PosLen),
            Some(sprinkles::progress::Message::suffix()),
        );

        let apps_pb = ProgressBar::new(handles.len() as u64)
            .with_message("Cleaning up")
            .with_style(pb_style.clone());
        mp.add(apps_pb.clone());

        let mut total_removed: u64 = 0;

        dbg!(handles.len());

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

                let size = get_recursive_size(&version_dir)?;

                total_removed += size;

                if !self.dry_run {
                    std::fs::remove_dir_all(version_dir)?;
                }

                pb.inc(1);
            }

            pb.finish();

            apps_pb.inc(1);
        }

        println!("Removed a total of {total_removed} bytes");

        Ok(())
    }
}
