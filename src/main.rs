use std::{
    ffi::OsString,
    fs::read_dir,
    io::{Error, Result},
    path::PathBuf,
};

use rayon::prelude::*;

use clap::Parser;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
struct SearchArgs {
    #[clap(help = "The pattern to search for (can be a regex)")]
    pattern: Regex,
}

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    /// The only thing we actually need
    version: String,
}

fn get_scoop_path() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| panic!("Could not find home directory"));

    home_dir.join("scoop")
}

fn main() -> Result<()> {
    let scoop_path = get_scoop_path();
    let scoop_buckets_path = scoop_path.join("buckets");

    let args = SearchArgs::parse();

    let scoop_buckets = read_dir(scoop_buckets_path)?.collect::<Result<Vec<_>>>()?;

    let mut matches = scoop_buckets
        .par_iter()
        .filter_map(|bucket| {
            let bucket_path = if bucket.path().join("bucket").exists() {
                bucket.path().join("bucket")
            } else {
                bucket.path()
            };

            let bucket_contents = read_dir(bucket_path)
                .unwrap()
                .collect::<Result<Vec<_>>>()
                .unwrap();

            let matches = bucket_contents
                .par_iter()
                .filter(|file| {
                    let path_raw = file.path();
                    let path = path_raw.as_os_str().to_string_lossy();

                    args.pattern.is_match(&path)
                })
                .map(|file| {
                    let path_raw = file.file_name();
                    let path = path_raw.as_os_str().to_string_lossy();

                    path.to_string()
                })
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
            println!("\n'{}' bucket:", bucket.to_string_lossy());

            old_bucket = bucket;
        }

        for mtch in matches {
            println!("  {}", mtch);
        }
    }

    Ok(())
}
