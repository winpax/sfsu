use std::{ffi::OsStr, time::UNIX_EPOCH};

use rayon::prelude::*;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{
    get_scoop_path,
    packages::{FromPath, InstallManifest, Manifest},
    IsTrue,
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
    #[clap(short, long, help = "The bucket to exclusively list in")]
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

        let offset = {
            let now = chrono::Local::now();

            *now.offset()
        };

        let outputs = read
            .par_iter()
            // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
            .filter(|package| package.path().iter().last() != Some(OsStr::new("scoop")))
            .map(|package| {
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

                    NaiveDateTime::from_timestamp_opt(secs, 0)
                        .expect("invalid or out-of-range datetime")
                };

                let date_time = DateTime::<FixedOffset>::from_local(naive_time, offset);

                let app_current = path.join("current");

                let manifest =
                    Manifest::from_path(app_current.join("manifest.json")).unwrap_or_default();

                let install_manifest = InstallManifest::from_path(app_current.join("install.json"))
                    .unwrap_or_default();

                anyhow::Ok(OutputPackage {
                    name: package_name.to_string(),
                    version: manifest.version,
                    source: install_manifest.get_source(),
                    updated: date_time.to_rfc3339(),
                    notes: if install_manifest.hold.is_true() {
                        String::from("Hold")
                    } else {
                        String::new()
                    },
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let max_lengths = outputs.iter().fold((0, 0, 0, 0, 0), |mut og, pkg| {
            use std::cmp;

            og.0 = cmp::max(pkg.name.len(), og.0);
            og.1 = cmp::max(pkg.version.len(), og.1);
            og.2 = cmp::max(pkg.source.len(), og.2);
            og.3 = cmp::max(pkg.updated.len(), og.3);
            og.4 = cmp::max(pkg.notes.len(), og.4);

            og
        });

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
