use std::{
    fs::read_dir,
    io::{Error, Result},
    path::PathBuf,
};

use rayon::prelude::*;

use clap::Parser;
use regex::Regex;

#[derive(Debug, Parser)]
struct SearchArgs {
    #[clap(help = "The pattern to search for (can be a regex)")]
    pattern: Regex,
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

    let matches = scoop_buckets
        .par_iter()
        .map(|bucket| {
            let bucket_path = if bucket.path().join("bucket").exists() {
                bucket.path().join("bucket")
            } else {
                bucket.path()
            };

            let bucket_contents = read_dir(bucket_path)?.collect::<Result<Vec<_>>>()?;

            let matches = bucket_contents
                .par_iter()
                .filter(|file| {
                    let path_raw = file.path();
                    let path = path_raw.as_os_str().to_string_lossy();

                    args.pattern.is_match(&path)
                })
                .map(|file| {
                    let path_raw = file.path();
                    let path = path_raw.as_os_str().to_string_lossy();

                    path.to_string()
                })
                .collect::<Vec<_>>();

            Ok::<_, Error>((bucket.file_name(), matches))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
