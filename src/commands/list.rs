use std::{path::Path, time::UNIX_EPOCH};

use rayon::prelude::*;

use chrono::NaiveDateTime;
use clap::Parser;
use colored::Colorize;
use quork::traits::truthy::ContainsTruth;
use serde::{Deserialize, Serialize};

use sfsu::{
    output::structured::Structured,
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
        let apps = sfsu::list_scoop_apps()?;

        let outputs = apps
            .par_iter()
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

            let values = outputs
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            let outputs =
                Structured::new(&["Name", "Version", "Source", "Updated", "Notes"], &values)
                    .with_max_length(30);

            print!("{outputs}");
        }

        Ok(())
    }
}

fn parse_package(path: impl AsRef<Path>) -> anyhow::Result<OutputPackage> {
    let path = path.as_ref();

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
            String::from("Held")
        } else {
            String::new()
        },
    })
}
