use std::fmt::Write;

use clap::Parser;
use colored::Colorize as _;
use parking_lot::Mutex;
use quork::prelude::*;
use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;

use sfsu::{
    buckets::Bucket,
    git::Repo,
    output::{
        sectioned::{Children, Section},
        structured::Structured,
    },
    packages::{install, outdated, Manifest},
    progress::style,
};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct StatusInfo {
    pub name: String,
    pub current: String,
    pub available: String,
    pub missing_dependencies: String,
    pub info: String,
}

impl StatusInfo {
    /// Get the outdated info from a local and remote manifest combo
    ///
    /// Returns [`None`] if they have the same version
    #[must_use]
    pub fn from_manifests(local: &Manifest, remote: &Manifest) -> Option<Self> {
        if local.version == remote.version {
            None
        } else {
            // Some(StatusInfo {
            //     name: remote.name.clone(),
            //     current: local.version.clone(),
            //     available: remote.version.clone(),
            // })
            todo!()
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,

    #[clap(from_global)]
    verbose: bool,
}

impl super::Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        #[derive(ListVariants)]
        enum Command {
            Scoop,
            Buckets,
            Packages,
        }

        let value = Mutex::new(Value::default());

        let pb = indicatif::ProgressBar::new(3).with_style(style(None, None));

        let outputs = Command::VARIANTS
            .into_par_iter()
            .map(|command| {
                let mut output = String::new();

                match command {
                    Command::Scoop => self.handle_scoop(&value, &mut output)?,
                    Command::Buckets => self.handle_buckets(&value, &mut output)?,
                    Command::Packages => self.handle_packages(&value, &mut output)?,
                };

                pb.inc(1);

                anyhow::Ok(output)
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        pb.finish_and_clear();

        if self.json {
            let output = serde_json::to_string(&value.lock().clone())?;
            println!("{output}");
        } else {
            for output in outputs {
                print!("{output}");
            }
        }

        Ok(())
    }
}

impl Args {
    fn handle_scoop(&self, value: &Mutex<Value>, output: &mut dyn Write) -> anyhow::Result<()> {
        let scoop_repo = Repo::scoop_app()?;

        let is_outdated = scoop_repo.outdated()?;

        if self.json {
            value.lock()["scoop"] = serde_json::to_value(is_outdated)?;
            return Ok(());
        } else if is_outdated {
            writeln!(
                output,
                "{}",
                "Scoop is out of date. Run `scoop update` to get the latest changes.".yellow()
            )?;
        } else {
            writeln!(output, "Scoop app is up to date.")?;
        }

        Ok(())
    }

    fn handle_buckets(&self, value: &Mutex<Value>, output: &mut dyn Write) -> anyhow::Result<()> {
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
                value.lock()["buckets"] = serde_json::to_value(&outdated_buckets)?;
                return Ok(());
            }

            if outdated_buckets.is_empty() {
                writeln!(output, "All buckets up to date.")?;
            } else {
                let title = format!("{} outdated buckets:", outdated_buckets.len());

                let section = Section::new(Children::from(outdated_buckets)).with_title(title);

                writeln!(output, "{section}")?;
            }
        } else {
            let buckets_outdated = buckets.par_iter().any(|bucket| {
                bucket.outdated().unwrap_or_else(|err| {
                    eprintln!("Failed to check bucket: {}", bucket.name());
                    error!(
                        "Failed to check bucket: {}. Threw Error: {:?}",
                        bucket.name(),
                        err
                    );
                    false
                })
            });

            if buckets_outdated {
                writeln!(
                    output,
                    "{}",
                    "Bucket(s) are out of date. Run `scoop update` to get the latest changes."
                        .yellow()
                )?;
            } else {
                writeln!(output, "All buckets up to date.")?;
            }
        }

        Ok(())
    }

    fn handle_packages(&self, value: &Mutex<Value>, output: &mut dyn Write) -> anyhow::Result<()> {
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
            value.lock()["packages"] = serde_json::to_value(&outdated)?;
            return Ok(());
        }

        if outdated.is_empty() {
            writeln!(output, "No outdated packages.")?;
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

            write!(output, "{outputs}")?;
            // }
        }

        Ok(())
    }
}
