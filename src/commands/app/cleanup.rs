use anyhow::Context;
use clap::Parser;
use futures::future;
use rayon::prelude::*;
use sprinkles::{
    contexts::ScoopContext,
    git::implementations::gix::date::time::format,
    handles::packages::PackageHandle,
    packages::{reference::package, CreateManifest, Manifest},
    progress::indicatif::{MultiProgress, ProgressBar},
};

#[derive(Debug, Clone, Parser)]
/// Clean up apps by removing old versions
pub struct Args {
    #[clap(short, long, help = "Cleanup all apps")]
    all: bool,

    #[clap(help = "The apps to cleanup")]
    apps: Vec<package::Reference>,
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

                // TODO: Remove this before release
                dbg!(&version_dir);
                #[cfg(debug_assertions)]
                std::thread::sleep(std::time::Duration::from_millis(2000));

                std::fs::remove_dir_all(version_dir)?;

                pb.inc(1);
            }

            pb.finish();

            apps_pb.inc(1);
        }

        todo!()
    }
}
