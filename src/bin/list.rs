use std::time::UNIX_EPOCH;

use rayon::prelude::*;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sfst::{
    buckets::Bucket,
    get_scoop_path,
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

            let naive_time = NaiveDateTime::from_timestamp(updated.try_into()?, 0);

            let date_time = DateTime::<FixedOffset>::from_local(naive_time, offset);

            let app_current = path.join("current");

            let manifest = Manifest::from_path(&app_current.join("manifest.json"))?;

            let install_manifest = InstallManifest::from_path(&app_current.join("install.json"))?;

            let bucket = Bucket::open(install_manifest.bucket)?;

            anyhow::Ok(OutputPackage {
                name: package_name.to_string(),
                version: manifest.version,
                source: bucket.get_remote()?,
                updated: date_time.to_rfc3339(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("{json}", json = serde_json::to_string_pretty(&outputs)?);

    Ok(())
}
