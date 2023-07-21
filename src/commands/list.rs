use std::{ffi::OsStr, fs::DirEntry, time::UNIX_EPOCH};

use rayon::prelude::*;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use colored::Colorize;
use quork::traits::truthy::ContainsTruth;
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
            let [nwidth, vwidth, swidth, uwidth, nowidth] =
                outputs.iter().fold([0, 0, 0, 0, 0], check_lengths);

            println!(
                "{:nwidth$} | {:vwidth$} | {:swidth$} | {:uwidth$} | {:nowidth$}",
                "Name", "Version", "Source", "Updated", "Notes",
            );

            for pkg in outputs {
                println!(
                    "{:nwidth$} | {:vwidth$} | {:swidth$} | {:uwidth$} | {:nowidth$}",
                    pkg.name, pkg.version, pkg.source, pkg.updated, pkg.notes,
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
    let updated = {
        let updated_sys = path.metadata()?.modified()?;

        updated_sys.duration_since(UNIX_EPOCH)?.as_secs()
    };

    let package_name = path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy();

    let naive_time = {
        let secs = updated.try_into()?;

        NaiveDateTime::from_timestamp_opt(secs, 0).expect("invalid or out-of-range datetime")
    };

    let offset = *chrono::Local::now().offset();

    let date_time = DateTime::<FixedOffset>::from_local(naive_time, offset);

    let app_current = path.join("current");

    let manifest = Manifest::from_path(app_current.join("manifest.json")).unwrap_or_default();

    let install_manifest =
        InstallManifest::from_path(app_current.join("install.json")).unwrap_or_default();

    anyhow::Ok(OutputPackage {
        name: package_name.to_string(),
        version: manifest.version,
        source: install_manifest.get_source(),
        updated: date_time.to_rfc3339(),
        notes: if install_manifest.hold.contains_truth() {
            String::from("Hold")
        } else {
            String::new()
        },
    })
}
