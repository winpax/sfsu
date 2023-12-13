use std::io::Error;

use colored::Colorize;
use itertools::Itertools;
use rayon::prelude::*;

use clap::{Parser, ValueEnum};
use regex::Regex;

use sfsu::{
    buckets::Bucket,
    output::sectioned::{Children, Section, Sections, Text},
    packages::{manifest::StringOrArrayOfStringsOrAnArrayOfArrayOfStrings, SearchMode},
};

use sfsu::packages::{is_installed, Manifest};
use strum::Display;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The regex pattern to search for, using Rust Regex syntax")]
    pattern: String,

    #[clap(
        short,
        long,
        help = "Whether or not the pattern should match case-sensitively"
    )]
    case_sensitive: bool,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(short, long, help = "Only search installed packages")]
    installed: bool,

    #[clap(short, long, help = "Search mode to use", default_value_t)]
    mode: SearchMode,
}

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        let (bucket, raw_pattern) = if self.pattern.contains('/') {
            let mut split = self.pattern.splitn(2, '/');

            // Bucket flag overrides bucket/package syntax
            let bucket = self.bucket.unwrap_or(split.next().unwrap().to_string());
            let pattern = split.next().unwrap();

            (Some(bucket), pattern.to_string())
        } else {
            (self.bucket, self.pattern)
        };

        let pattern = {
            Regex::new(&format!(
                "{}{raw_pattern}",
                if self.case_sensitive { "" } else { "(?i)" },
            ))
            .expect("Invalid Regex provided. See https://docs.rs/regex/latest/regex/ for more info")
        };

        let matching_buckets: Vec<Bucket> = if let Some(Ok(bucket)) = bucket.map(Bucket::new) {
            vec![bucket]
        } else {
            Bucket::list_all()?
        };

        let mut matches = matching_buckets
            .par_iter()
            .filter_map(|bucket| bucket.matches(&pattern, self.mode))
            .collect::<Result<Vec<_>, _>>()?;

        matches.par_sort_by_key(|x| x.title.clone());

        print!("{}", Sections::from_vec(matches));

        Ok(())
    }
}
