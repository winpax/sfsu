use std::{fs::read_dir, io::Result, path::PathBuf};

use clap::Parser;
use regex::Regex;

#[derive(Debug, Parser)]
struct SearchArgs {
    #[clap(help = "The pattern to search for")]
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

    for bucket in scoop_buckets {
        let bucket_path = if bucket.path().join("bucket").exists() {
            bucket.path().join("bucket")
        } else {
            bucket.path()
        };

        let bucket_contents = read_dir(bucket_path)?.collect::<Result<Vec<_>>>()?;

        let matches = bucket_contents.iter().filter(|file| {
            let path_raw = file.path();
            let path = path_raw.as_os_str().to_string_lossy();

            args.pattern.is_match(&path)
        });
    }

    Ok(())
}
