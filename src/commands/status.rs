use clap::Parser;
use colored::Colorize as _;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use serde_json::Value;

use sfsu::{
    buckets::Bucket,
    git::Repo,
    output::{
        sectioned::{Children, Section},
        structured::Structured,
    },
    packages::{install, outdated},
    progress::{style, ProgressOptions},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,

    #[clap(from_global)]
    verbose: bool,
}

impl super::Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        let mut value = Value::default();

        self.handle_scoop(&mut value)?;
        if !self.json {
            println!();
        }
        self.handle_buckets(&mut value)?;
        if !self.json {
            println!();
        }
        self.handle_packages(&mut value)?;

        if self.json {
            let output = serde_json::to_string(&value)?;
            println!("{output}");
        }

        Ok(())
    }
}

impl Args {
    fn handle_scoop(&self, value: &mut Value) -> anyhow::Result<()> {
        let scoop_repo = Repo::scoop_app()?;

        let is_outdated = scoop_repo.outdated()?;

        if self.json {
            value["scoop"] = serde_json::to_value(is_outdated)?;
            return Ok(());
        } else if is_outdated {
            eprintln!(
                "{}",
                "Scoop is out of date. Run `scoop update` to get the latest changes.".yellow()
            );
        } else {
            eprintln!("Scoop app is up to date.");
        }

        Ok(())
    }

    fn handle_buckets(&self, value: &mut Value) -> anyhow::Result<()> {
        let buckets = Bucket::list_all()?;

        // Handle buckets
        if self.verbose || self.json {
            let outdated_buckets = buckets
                .par_iter()
                .filter_map(|bucket| {
                    bucket.outdated().ok().and_then(|outdated| {
                        if outdated {
                            Some(bucket.name().to_string())
                        } else {
                            None
                        }
                    })
                })
                .collect::<Vec<_>>();

            if self.json {
                value["buckets"] = serde_json::to_value(&outdated_buckets)?;
                return Ok(());
            }

            if outdated_buckets.is_empty() {
                eprintln!("All buckets up to date.");
            } else {
                let title = format!("{} outdated buckets:", outdated_buckets.len());

                let section = Section::new(Children::from(outdated_buckets)).with_title(title);

                println!("{section}");
            }
        } else {
            let buckets_outdated = buckets.par_iter().any(|bucket| {
                bucket.outdated().unwrap_or_else(|_| {
                    eprintln!("Failed to check bucket: {}", bucket.name());
                    false
                })
            });

            if buckets_outdated {
                eprintln!(
                    "{}",
                    "Bucket(s) are out of date. Run `scoop update` to get the latest changes."
                        .yellow()
                );
            } else {
                eprintln!("All buckets up to date.");
            }
        }

        Ok(())
    }

    fn handle_packages(&self, value: &mut Value) -> anyhow::Result<()> {
        let apps = install::Manifest::list_all_unchecked()?;

        let mut outdated: Vec<outdated::Info> = apps
            .par_iter()
            .flat_map(|app| -> anyhow::Result<outdated::Info> {
                if let Some(bucket) = &app.bucket {
                    let local_manifest = app.get_manifest()?;
                    // TODO: Add the option to check all buckets and find the highest version (will require semver to order versions)
                    let bucket = Bucket::from_name(bucket)?;

                    let remote_manifest = bucket.get_manifest(&app.name)?;

                    if let Some(info) =
                        outdated::Info::from_manifests(&local_manifest, &remote_manifest)
                    {
                        Ok(info)
                    } else {
                        anyhow::bail!("no update available")
                    }
                } else {
                    anyhow::bail!("no bucket specified")
                }
            })
            .collect();

        if self.json {
            outdated.dedup();
            value["packages"] = serde_json::to_value(&outdated)?;
            return Ok(());
        }

        if outdated.is_empty() {
            println!("No outdated packages.");
        } else {
            outdated.dedup();
            outdated.par_sort_by(|a, b| a.name.cmp(&b.name));

            let values = outdated
                .par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            // if self.json {
            //     let output = serde_json::to_string_pretty(&values)?;

            //     println!("{output}");
            // } else {
            // TODO: Add a better way to add colours than this
            // TODO: p.s this doesnt work atm
            // use colored::Colorize;
            // let values = values
            //     .into_par_iter()
            //     .map(|mut value| {
            //         if let Some(current) = value.get_mut("Current") {
            //             *current = current.as_str().unwrap().red().to_string().into();
            //         }

            //         if let Some(available) = value.get_mut("Available") {
            //             *available = available.as_str().unwrap().green().to_string().into();
            //         }

            //         value
            //     })
            //     .collect::<Vec<_>>();

            let outputs =
                Structured::new(&["Name", "Current", "Available"], &values).with_max_length(30);

            print!("{outputs}");
            // }
        }

        Ok(())
    }
}
