use std::{process::Command, time::UNIX_EPOCH};

use rayon::prelude::*;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sfst::{
    get_powershell_path, get_scoop_path,
    packages::{FromPath, InstallManifest, Manifest},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OutputPackage {
    name: String,
    version: String,
    source: String,
    updated: String,
}

#[derive(Debug, Parser)]
struct ListArgs {
    #[clap(help = "The pattern to search for (can be a regex)")]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(long, help = "Print in a more human readable format, rather than JSON")]
    human: bool,
}

fn main() -> anyhow::Result<()> {
    let args = ListArgs::parse();

    let scoop_apps_path = get_scoop_path().join("apps");

    let read = scoop_apps_path.read_dir()?.collect::<Result<Vec<_>, _>>()?;

    let offset = {
        let now = chrono::Local::now();

        *now.offset()
    };

    let outputs = read
        .par_iter()
        // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
        // TODO: More efficient way to do this check?
        .filter(|package| {
            package
                .path()
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                != "scoop"
        })
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

            let manifest = Manifest::from_path(&app_current.join("manifest.json"))?;

            let install_manifest = InstallManifest::from_path(&app_current.join("install.json"))?;

            anyhow::Ok(OutputPackage {
                name: package_name.to_string(),
                version: manifest.version,
                source: install_manifest.bucket,
                updated: date_time.to_rfc3339(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let max_lengths = outputs.iter().fold((0, 0, 0, 0), |mut og, pkg| {
        use std::cmp;

        og.0 = cmp::min(pkg.name.len(), og.0);
        og.1 = cmp::min(pkg.version.len(), og.1);
        og.2 = cmp::min(pkg.source.len(), og.2);
        og.3 = cmp::min(pkg.updated.len(), og.3);

        og
    });

    if args.human {
        println!("Name | Version | Source | Updated");

        for pkg in outputs {
            println!(
                "{:nwidth$} | {:vwidth$} | {:swidth$} | {:uwidth$}",
                pkg.name,
                pkg.version,
                pkg.source,
                pkg.updated,
                nwidth = max_lengths.0,
                vwidth = max_lengths.1,
                swidth = max_lengths.2,
                uwidth = max_lengths.3
            );
        }
    } else {
        let output_json = serde_json::to_string_pretty(&outputs)?;

        println!("{output_json}");
    }

    Ok(())
}
