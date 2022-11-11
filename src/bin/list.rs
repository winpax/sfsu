use std::{os::windows::prelude::MetadataExt, process::Command};

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sfst::{
    buckets::Bucket,
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

    #[clap(long, help = "Print in JSON format rather than Powershell format")]
    json: bool,

    #[clap(long, help = "Print the Powershell hook")]
    hook: bool,
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
        .iter()
        .map(|package| {
            let path = dunce::realpath(package.path())?;
            let updated = path.metadata()?.last_write_time();

            let package_name = path
                .components()
                .last()
                .unwrap()
                .as_os_str()
                .to_string_lossy();

            let naive_time = NaiveDateTime::from_timestamp(updated.try_into()?, 0);

            let date_time = DateTime::<FixedOffset>::from_local(naive_time, offset);

            let manifest = Manifest::from_path(&path)?;
            let install_manifest = InstallManifest::from_path(&path)?;

            let bucket = Bucket::open(install_manifest.bucket)?;

            anyhow::Ok(OutputPackage {
                name: package_name.to_string(),
                version: manifest.version,
                source: bucket.get_remote()?,
                updated: date_time.to_rfc3339(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if args.json {
        let output_json = serde_json::to_string_pretty(&outputs)?;

        println!("{output_json}");
    } else {
        let output = serde_json::to_string(&outputs)?;

        let pwsh_path = get_powershell_path()?;
        let cmd = Command::new(pwsh_path);
    }

    Ok(())
}
