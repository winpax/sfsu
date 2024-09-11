use std::collections::HashMap;

use clap::Parser;
use dialoguer::Confirm;
use rayon::iter::IntoParallelIterator;
use sprinkles::{
    contexts::ScoopContext,
    packages::reference::{manifest, package},
    progress::{indicatif::ProgressBar, style},
};

use crate::output::colours::{eprintln_yellow, yellow};

#[derive(Debug, Clone, Parser)]
/// Purge package's persist folder
pub struct Args {
    #[clap(help = "The package to purge")]
    apps: Vec<package::Reference>,

    #[clap(from_global)]
    assume_yes: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> anyhow::Result<()> {
        let purging_uninstalled = self.apps.is_empty();
        let apps = if purging_uninstalled {
            let references = list_uninstalled(ctx)?;
            references
                .into_iter()
                .map(manifest::Reference::into_package_ref)
                .collect()
        } else {
            self.apps
        }
        .into_iter()
        .map(|reference| reference.first(ctx).unwrap());

        let mut app_paths = HashMap::new();
        for app in apps {
            let reference = unsafe {
                manifest::Reference::BucketNamePair {
                    bucket: app.bucket().to_owned(),
                    name: app.name().to_owned(),
                }
            };

            let persist_path = ctx.persist_path().join(unsafe { app.name() });

            if app_paths.contains_key(&reference) {
                continue;
            }

            if !persist_path.exists() {
                eprintln_yellow!("Persist folder does not exist for {}", unsafe {
                    app.name()
                });
                continue;
            }

            app_paths.insert(reference, (app, persist_path));
        }

        if purging_uninstalled {
            eprintln!("Purging persist folders for uninstalled apps:");
        } else {
            eprintln!("Purging persist folders for the following apps:");
        }
        for (app, persist_path) in app_paths.values() {
            eprintln!(
                "- {}/{} ({})",
                unsafe { app.bucket() },
                unsafe { app.name() },
                persist_path.display()
            );
        }
        eprintln!();

        if !self.assume_yes
            && !Confirm::new()
                .with_prompt(
                    yellow!(
                        "Are you sure you want to purge the persist folder for {}?",
                        if app_paths.len() == 1 {
                            "this app".to_string()
                        } else {
                            format!("{} apps", app_paths.len())
                        }
                    )
                    .to_string(),
                )
                .default(false)
                .interact()?
        {
            return Ok(());
        }

        if !self.assume_yes && app_paths
                .values()
                .any(|(app, _)| app.is_installed(ctx, None)) && !Confirm::new()
                .with_prompt(
                    yellow!(
                        "Some apps are installed. This could cause issues when running the app. Are you sure you want to continue?")
                    .to_string(),
                )
                .default(false)
                .interact()? {
            return Ok(())
        }

        if app_paths.len() == 1 {
            let (app, path) = app_paths.values().next().unwrap();

            eprintln_yellow!("Purging persist folder for {}", unsafe { app.name() });

            std::fs::remove_dir_all(path)?;
        } else {
            let pb = ProgressBar::new(app_paths.len() as u64).with_style(style(None, None));

            for (app, persist_path) in app_paths.values() {
                pb.set_message(format!("Purging persist folder for {}", unsafe {
                    app.name()
                }));
                pb.inc(1);
                std::fs::remove_dir_all(persist_path)?;
            }
        }

        Ok(())
    }
}

fn list_uninstalled(ctx: &impl ScoopContext) -> anyhow::Result<Vec<manifest::Reference>> {
    let persist_path = ctx.persist_path();

    let mut uninstalled = vec![];

    for entry in std::fs::read_dir(persist_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let Some(app_name) = path
                .file_name()
                .map(|file_name| file_name.to_string_lossy())
            else {
                continue;
            };

            let reference = manifest::Reference::Name(app_name.to_string());

            uninstalled.push(reference);
        }
    }

    Ok(uninstalled)
}
