use std::{
    ffi::OsStr,
    fs::{read_dir, DirEntry},
    io::Error,
    rc::Rc,
};

use colored::Colorize;
use rayon::prelude::*;

use clap::{Parser, ValueEnum};
use regex::Regex;

use sfsu::{buckets, packages::manifest::StringOrArrayOfStringsOrAnArrayOfArrayOfStrings};

use sfsu::packages::{is_installed, CreateManifest, Manifest};
use strum::Display;

#[derive(Debug, Default, Copy, Clone, ValueEnum, Display, Parser)]
#[strum(serialize_all = "snake_case")]
enum SearchMode {
    Binary,
    Name,
    #[default]
    Both,
}

#[derive(Debug, Clone, Parser)]
/// Search for a package
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

fn parse_output(
    file: &DirEntry,
    bucket: impl AsRef<str>,
    installed_only: bool,
    pattern: &Regex,
    mode: SearchMode,
) -> Option<String> {
    let path = file.path();

    let is_valid_extension = matches!(path.extension().and_then(OsStr::to_str), Some("json"));

    // This may be a bit of a hack, but it works
    let file_name = path
        .with_extension("")
        .file_name()
        .map(|osstr| osstr.to_string_lossy().to_string());
    let package_name = file_name.unwrap();

    match Manifest::from_path(file.path()) {
        Ok(manifest) => {
            let match_criteria = match_criteria(&package_name, &manifest, mode);
            let match_output = match_criteria(pattern.clone());
            let is_installed = is_installed(&package_name, Some(bucket));
            if installed_only {
                if is_installed {
                    Some(format!("{} ({})", package_name, manifest.version))
                } else {
                    None
                }
            } else {
                Some(format!(
                    "{} ({}) {}",
                    if package_name == pattern.to_string() {
                        package_name.bold().to_string()
                    } else {
                        package_name
                    },
                    manifest.version,
                    if is_installed { "[installed]" } else { "" },
                ))
            }
        }
        Err(_) => Some(format!("{package_name} - Invalid")),
    }
}

enum MatchOutput {
    FileName,
    BinaryName(String),
}

impl std::fmt::Display for MatchOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchOutput::FileName => Ok(()),
            MatchOutput::BinaryName(name) => write!(f, "{}", name.bold()),
        }
    }
}

fn match_criteria(
    file_name: &str,
    manifest: &Manifest,
    mode: SearchMode,
) -> impl FnOnce(Regex) -> Vec<MatchOutput> {
    // use std::rc::Rc;
    // let name = Rc::new(file_name);

    let binaries = manifest
        .bin
        .clone()
        .map(StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::to_vec)
        .unwrap_or_default();

    let file_name = file_name.to_string();

    move |pattern| {
        let mut binary_names: Vec<String> = vec![];
        let mut output = vec![];
        match mode {
            SearchMode::Binary => todo!(),
            SearchMode::Name => todo!(),
            SearchMode::Both => {
                if pattern.is_match(&file_name) {
                    output.push(MatchOutput::FileName);
                }

                output.extend(binary_names.iter().filter_map(|b| {
                    if pattern.is_match(b) {
                        Some(MatchOutput::BinaryName(b.clone()))
                    } else {
                        None
                    }
                }));

                output
            }
        }
    }
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
                "{}{}",
                if self.case_sensitive { "" } else { "(?i)" },
                &raw_pattern
            ))
            .expect("Invalid Regex provided. See https://docs.rs/regex/latest/regex/ for more info")
        };

        let all_scoop_buckets = buckets::Bucket::list_all()?;

        let scoop_buckets = {
            if let Some(bucket) = bucket {
                all_scoop_buckets
                    .into_iter()
                    .filter(|scoop_bucket| {
                        let path = scoop_bucket.path();
                        match path.components().last() {
                            Some(x) => x.as_os_str() == bucket.as_str(),
                            None => false,
                        }
                    })
                    .collect()
            } else {
                all_scoop_buckets
            }
        };

        let mut matches = scoop_buckets
            .par_iter()
            .filter_map(|bucket| {
                // Ignore loose files in the buckets dir
                if !bucket.path().is_dir() {
                    return None;
                }

                let bucket_path = {
                    let bk_path = bucket.path().join("bucket");

                    if bk_path.exists() {
                        bk_path
                    } else {
                        bucket.path()
                    }
                };

                let bucket_contents = read_dir(bucket_path)
                    .and_then(Iterator::collect::<Result<Vec<_>, _>>)
                    .unwrap();

                let matches = bucket_contents
                    .par_iter()
                    .filter_map(|file| {
                        parse_output(file, &bucket.name, self.installed, &pattern, self.mode)
                    })
                    .collect::<Vec<_>>();

                if matches.is_empty() {
                    None
                } else {
                    Some(Ok::<_, Error>((bucket.name.clone(), matches)))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        matches.par_sort_by_key(|x| x.0.clone());

        let mut old_bucket = String::new();

        for (bucket, matches) in matches {
            if bucket != old_bucket {
                // Do not print the newline on the first bucket
                if !old_bucket.is_empty() {
                    println!();
                }

                println!("'{bucket}' bucket:");

                old_bucket = bucket;
            }

            for package in matches {
                println!("  {package}");
            }
        }

        Ok(())
    }
}
