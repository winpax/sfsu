use std::{
    ffi::OsString,
    fs::{read_dir, DirEntry, File},
    io::{Error, Read, Result},
    path::{Path, PathBuf},
};

use rayon::prelude::*;

use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct SearchArgs {
    #[clap(help = "The pattern to search for (can be a regex)")]
    pattern: Option<Regex>,

    #[clap(long, help = "Print the Powershell hook")]
    hook: bool,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    /// The version of the package
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct InstallManifest {
    /// The bucket the package was installed from
    bucket: String,
}

fn get_scoop_path() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| panic!("Could not find home directory"));

    home_dir.join("scoop")
}

// TODO: Add installed marker

fn parse_output(file: &DirEntry, bucket: impl AsRef<str>) -> String {
    // This may be a bit of a hack, but it works
    let path = file.path().with_extension("");
    let file_name = path.file_name();
    let package = file_name.unwrap().to_string_lossy().to_string();

    let mut buf = String::new();

    File::open(file.path())
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();

    let manifest: Manifest = serde_json::from_str(&buf).unwrap();

    format!(
        "{} ({}) {}",
        package,
        manifest.version,
        if is_installed(&package, bucket) {
            "[Installed]"
        } else {
            ""
        }
    )
}

fn is_installed(manifest_name: impl AsRef<Path>, bucket: impl AsRef<str>) -> bool {
    let scoop_path = get_scoop_path();
    let installed_path = scoop_path
        .join("apps")
        .join(manifest_name)
        .join("current/install.json");

    if installed_path.exists() {
        let mut buf = String::new();

        File::open(installed_path)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();

        let manifest: InstallManifest = serde_json::from_str(&buf).unwrap();

        manifest.bucket == bucket.as_ref()
    } else {
        false
    }
}

fn main() -> Result<()> {
    let scoop_path = get_scoop_path();
    let scoop_buckets_path = scoop_path.join("buckets");

    let args = SearchArgs::parse();

    if args.hook {
        const HOOK: &str = r#"function scoop { if ($args[0] -eq "search") { sfss.exe @($args | Select-Object -Skip 1) } else { scoop.ps1 @args } }"#;
        print!("{HOOK}");
        return Ok(());
    }

    let pattern = args.pattern.expect("No pattern provided");

    if let Some(bucket) = args.bucket {
        let path = {
            let bk_base = scoop_buckets_path.join(&bucket);
            let bk_path = bk_base.join("bucket");

            if bk_path.exists() {
                bk_path
            } else {
                bk_base
            }
        };

        let manifests = read_dir(path)?
            .filter_map(|file| {
                if let Ok(file) = file {
                    if pattern.is_match(&file.path().to_string_lossy()) {
                        return Some(parse_output(&file, &bucket));
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        if manifests.is_empty() {
            println!("Did not find any matching manifests in bucket '{}'", bucket);
        } else {
            println!("Found in bucket '{}':", bucket);

            for manifest in manifests {
                println!("  {}", manifest);
            }
        }
    }

    let scoop_buckets = read_dir(scoop_buckets_path)?.collect::<Result<Vec<_>>>()?;

    let mut matches = scoop_buckets
        .par_iter()
        .filter_map(|bucket| {
            let bucket_path = {
                let bk_path = bucket.path().join("bucket");

                if bk_path.exists() {
                    bk_path
                } else {
                    bucket.path()
                }
            };

            let bucket_contents = read_dir(bucket_path)
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap();

            let matches = bucket_contents
                .par_iter()
                .filter(|file| {
                    let path_raw = file.path();
                    let path = path_raw.to_string_lossy();

                    pattern.is_match(&path)
                })
                .map(|x| parse_output(x, bucket.file_name().to_string_lossy()))
                .collect::<Vec<_>>();

            if matches.is_empty() {
                None
            } else {
                Some(Ok::<_, Error>((bucket.file_name(), matches)))
            }
        })
        .collect::<Result<Vec<_>>>()?;

    matches.par_sort_by_key(|x| x.0.clone());

    let mut old_bucket = OsString::new();

    for (bucket, matches) in matches {
        if bucket != old_bucket {
            // Do not print the newline on the first bucket
            if old_bucket != OsString::new() {
                println!();
            }

            println!("'{}' bucket:", bucket.to_string_lossy());

            old_bucket = bucket;
        }

        for mtch in matches {
            println!("  {}", mtch);
        }
    }

    Ok(())
}
