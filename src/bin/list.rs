use std::time::UNIX_EPOCH;

use rayon::prelude::*;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use regex::Regex;
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
    #[serde(skip_serializing)]
    bucket_name: String,
    updated: String,
}

#[derive(Debug, Parser)]
struct ListArgs {
    #[clap(help = "The pattern to search for (can be a regex)")]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(
        long,
        help = "Print the output as json, rather than a human readable format"
    )]
    json: bool,
}

fn main() -> anyhow::Result<()> {
    let args = ListArgs::parse();

    let scoop_apps_path = get_scoop_path().join("apps");

    let read = scoop_apps_path.read_dir()?.collect::<Result<Vec<_>, _>>()?;

    let offset = {
        let now = chrono::Local::now();

        *now.offset()
    };

    let pattern = args.pattern.unwrap_or_else(|| ".*".to_string());
    let name_regex = Regex::new(&pattern)?;

    let outputs = read
        .par_iter()
        // We cannot search the scoop app as it is built in and hence doesn't contain any manifest
        // Additionally don't search any apps that don't match the pattern
        // TODO: More efficient way to do this check?
        .filter(|package| {
            let name = {
                let path = package.path();
                let components = path.components();

                components
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy()
                    .to_string()
            };

            name != "scoop" && name_regex.is_match(&name)
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

            let bucket = Bucket::open(&install_manifest.bucket)?;

            anyhow::Ok(OutputPackage {
                name: package_name.to_string(),
                version: manifest.version,
                source: bucket.get_remote()?,
                bucket_name: install_manifest.bucket,
                updated: date_time.to_rfc3339(),
            })
        })
        // Remove results that do not match the bucket
        .filter(|f| match (f, &args.bucket) {
            (Ok(_), None) => true,
            (Ok(f), Some(ref bucket)) => f.bucket_name == *bucket,
            (Err(_), _) => false,
        })
        .collect::<Result<Vec<_>, _>>()?;

    println!("{json}", json = serde_json::to_string_pretty(&outputs)?);

    Ok(())
}
