use std::{ffi::OsStr, fs::DirEntry, time::UNIX_EPOCH};

use rayon::prelude::*;

use chrono::NaiveDateTime;
use clap::Parser;
use colored::Colorize;
use quork::{traits::truthy::ContainsTruth, truncate::Truncate};
use serde::{Deserialize, Serialize};

use sfsu::{
    get_scoop_path,
    packages::{CreateManifest, InstallManifest, Manifest},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OutputPackage {
    name: String,
    version: String,
    source: String,
    updated: String,
    notes: String,
}

#[derive(Debug, Clone, Parser)]
/// List all installed packages
pub struct Args {
    #[clap(
        help = format!("The pattern to search for (can be a regex). {}", "DEPRECATED: Use sfsu search --installed".yellow())
    )]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively list packages in")]
    bucket: Option<String>,

    #[clap(
        long,
        help = "Print in the raw JSON output, rather than a human readable format"
    )]
    json: bool,
}

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        let scoop_apps_path = get_scoop_path().join("apps");

        let read = scoop_apps_path.read_dir()?.collect::<Result<Vec<_>, _>>()?;

        let outputs = read
            .par_iter()
            // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
            .filter(|package| package.path().iter().last() != Some(OsStr::new("scoop")))
            .map(parse_package)
            .filter(|package| {
                if let Ok(pkg) = package {
                    if let Some(ref bucket) = self.bucket {
                        return &pkg.source == bucket;
                    }
                }
                // Keep errors so that the following line will return the error
                true
            })
            .collect::<Result<Vec<_>, _>>()?;

        if self.json {
            let output_json = serde_json::to_string_pretty(&outputs)?;

            println!("{output_json}");
        } else {
            if outputs.is_empty() {
                println!("No packages found.");
                return Ok(());
            }

            #[allow(clippy::similar_names)]
            let [name_width, version_width, source_width, updated_width, notes_width] =
                outputs.iter().fold([0, 0, 0, 0, 0], check_lengths);

            println!(
                "{:name_width$} | {:version_width$} | {:source_width$} | {:updated_width$} | {:notes_width$}",
                "Name", "Version", "Source", "Updated", "Notes",
            );

            for pkg in outputs {
                println!(
                    "{:name_width$} | {:version_width$} | {:source_width$} | {:updated_width$} | {:notes_width$}",
                    Truncate::new(pkg.name, name_width).with_suffix("..."), pkg.version, pkg.source, pkg.updated, pkg.notes,
                );
            }
        }

        Ok(())
    }
}

fn check_lengths(og: [usize; 5], pkg: &OutputPackage) -> [usize; 5] {
    // Checks for the largest size out of the previous one, the current one and the section title
    // Note that all widths use "Updated" as it is the longest section title
    let default_width = "Updated".len();

    og.map(|element| {
        *[default_width, pkg.updated.len(), element]
            .iter()
            .max()
            .unwrap_or(&default_width)
    })
}

fn parse_package(package: &DirEntry) -> anyhow::Result<OutputPackage> {
    let path = dunce::realpath(package.path())?;

    let package_name = path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy();

    let naive_time = {
        let updated = {
            let updated_sys = path.metadata()?.modified()?;

            updated_sys.duration_since(UNIX_EPOCH)?.as_secs()
        };

        NaiveDateTime::from_timestamp_opt(updated.try_into()?, 0)
            .expect("invalid or out-of-range datetime")
    };

    let app_current = path.join("current");

    let manifest = Manifest::from_path(app_current.join("manifest.json")).unwrap_or_default();

    let install_manifest =
        InstallManifest::from_path(app_current.join("install.json")).unwrap_or_default();

    anyhow::Ok(OutputPackage {
        name: package_name.to_string(),
        version: manifest.version,
        source: install_manifest.get_source(),
        updated: naive_time.to_string(),
        notes: if install_manifest.hold.contains_truth() {
            String::from("Hold")
        } else {
            String::new()
        },
    })
}
