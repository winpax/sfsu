use std::{ffi::OsStr, fs::DirEntry, time::UNIX_EPOCH};

use rayon::prelude::*;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use quork::traits::truthy::ContainsTruth;
use serde::{Deserialize, Serialize};

use crate::{
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
    #[clap(short, long, help = "The bucket to exclusively list packages in")]
    bucket: Option<String>,

    #[clap(
        long,
        help = "Print in the raw JSON output, rather than a human readable format"
    )]
    json: bool,
}

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
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

        let max_lengths = outputs.iter().fold((0, 0, 0, 0, 0), check_lengths);

        if self.json {
            let output_json = serde_json::to_string_pretty(&outputs)?;

            println!("{output_json}");
        } else {
            println!(
                "{:nwidth$} | {:vwidth$} | {:swidth$} | {:uwidth$} | {:nowidth$}",
                "Name",
                "Version",
                "Source",
                "Updated",
                "Notes",
                nwidth = max_lengths.0,
                vwidth = max_lengths.1,
                swidth = max_lengths.2,
                uwidth = max_lengths.3,
                nowidth = max_lengths.4,
            );

            for pkg in outputs {
                println!(
                    "{:nwidth$} | {:vwidth$} | {:swidth$} | {:uwidth$} | {:nowidth$}",
                    pkg.name,
                    pkg.version,
                    pkg.source,
                    pkg.updated,
                    pkg.notes,
                    nwidth = max_lengths.0,
                    vwidth = max_lengths.1,
                    swidth = max_lengths.2,
                    uwidth = max_lengths.3,
                    nowidth = max_lengths.4,
                );
            }
        }

        Ok(())
    }
}

fn offset() -> FixedOffset {
    let now = chrono::Local::now();

    *now.offset()
}

fn check_lengths(
    og: (usize, usize, usize, usize, usize),
    pkg: &OutputPackage,
) -> (usize, usize, usize, usize, usize) {
    let mut new = og;
    // Checks for the largest size out of the previous one, the current one and the section title
    new.0 = *["Name".len(), pkg.name.len(), new.0].iter().max().unwrap();
    new.1 = *["Version".len(), pkg.version.len(), new.1]
        .iter()
        .max()
        .unwrap();
    new.2 = *["Source".len(), pkg.source.len(), new.2]
        .iter()
        .max()
        .unwrap();
    new.3 = *["Updated".len(), pkg.updated.len(), new.3]
        .iter()
        .max()
        .unwrap();
    new.4 = *["Notes".len(), pkg.notes.len(), new.4]
        .iter()
        .max()
        .unwrap();

    new
}

fn parse_package(package: &DirEntry) -> anyhow::Result<OutputPackage> {
    let path = dunce::realpath(package.path())?;
    let updated_sys = path.metadata()?.modified()?;
    let updated = updated_sys.duration_since(UNIX_EPOCH)?.as_secs();

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

    let date_time = DateTime::<FixedOffset>::from_local(naive_time, offset());

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
