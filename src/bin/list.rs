use std::{fs::DirEntry, os::windows::prelude::MetadataExt};

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sfst::buckets::{self, Bucket};

#[derive(Debug, Serialize, Deserialize)]
struct OutputBucket {
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

    #[clap(long, help = "Print the Powershell hook")]
    hook: bool,
}

fn main() -> anyhow::Result<()> {
    let scoop_buckets_path = buckets::get_path();

    let read = scoop_buckets_path
        .read_dir()?
        .collect::<Result<Vec<DirEntry>, _>>()?;

    let offset = {
        let now = chrono::Local::now();

        *now.offset()
    };

    for bucket in read {
        let path = bucket.path();
        let updated = path.metadata()?.last_write_time();

        let naive_time = NaiveDateTime::from_timestamp(updated.try_into()?, 0);

        let date_time = DateTime::<FixedOffset>::from_local(naive_time, offset);

        let bucket = Bucket::open(path)?;

        OutputBucket {
            name: bucket.name().to_string(),
            version: bucket.version().to_string(),
            source: bucket.source().to_string(),
            updated: date_time.unwrap().to_string(),
        }
    }

    Ok(())
}
