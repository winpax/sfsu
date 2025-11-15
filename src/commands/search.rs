use std::collections::HashMap;

use itertools::Itertools;
use rayon::prelude::*;

use clap::Parser;
use regex::Regex;

use sprinkles::{
    Architecture, buckets::Bucket, contexts::ScoopContext, packages::SearchMode, version::Version,
};

use crate::{
    calm_panic::CalmUnwrap,
    output::sectioned::{Children, Section, Sections, Text},
    searching::MatchedManifest,
};

impl MatchedManifest {
    fn to_section(&self) -> Section<Text<String>> {
        let styled_package_name = if self.exact_match() {
            console::style(unsafe { self.manifest().name() })
                .bold()
                .to_string()
        } else {
            unsafe { self.manifest().name() }.to_string()
        };

        let installed_text = if self.installed() { "[installed] " } else { "" };

        let title = format!(
            "{styled_package_name} ({}) {installed_text}",
            self.manifest().version
        );

        if self.bins().is_empty() {
            Section::new(Children::None)
        } else {
            let bins = self
                .bins()
                .iter()
                .map(|output| {
                    Text::new(format!(
                        "{}{}",
                        crate::output::WHITESPACE,
                        console::style(output).bold()
                    ))
                })
                .collect_vec();

            Section::new(Children::from(bins))
        }
        .with_title(title)
    }

    fn into_output(self) -> MatchedOutput {
        MatchedOutput {
            name: unsafe { self.manifest().name() }.to_string(),
            bucket: unsafe { self.manifest().bucket() }.to_string(),
            version: self.manifest().version.clone(),
            installed: self.installed(),
            bins: self.bins().clone(),
        }
    }
}

impl std::fmt::Display for MatchedManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.to_section(), f)
    }
}

#[derive(Debug, serde::Serialize)]
struct MatchedOutput {
    name: String,
    bucket: String,
    version: Version,
    installed: bool,
    bins: Vec<String>,
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

    #[clap(from_global)]
    arch: Architecture,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let (bucket, raw_pattern) =
            if let Some((bucket, raw_pattern)) = self.pattern.split_once('/') {
                warn!("bucket/package syntax is deprecated. Please use the --bucket flag instead");
                (
                    Some({
                        // Bucket flag overrides bucket/package syntax
                        if let Some(bucket) = self.bucket {
                            warn!("Using bucket flag instead of bucket/package syntax");
                            bucket
                        } else {
                            bucket.to_string()
                        }
                    }),
                    raw_pattern.to_string(),
                )
            } else {
                (self.bucket, self.pattern)
            };

        let pattern = {
            Regex::new(&format!(
                "{}{raw_pattern}",
                if self.case_sensitive { "" } else { "(?i)" },
            ))
            .calm_expect(
                "Invalid Regex provided. See https://docs.rs/regex/latest/regex/ for more info",
            )
        };

        let matching_buckets: Vec<Bucket> = match bucket.map(|name| Bucket::from_name(ctx, name)) {
            Some(Ok(bucket)) => vec![bucket],
            _ => Bucket::list_all(ctx)?,
        };

        let buckets: HashMap<String, Vec<MatchedManifest>> = matching_buckets
            .par_iter()
            .filter_map(
                |bucket| match bucket.matches(ctx, self.installed, &pattern, self.mode) {
                    Ok(manifests) => {
                        let matches = manifests
                            .into_par_iter()
                            .map(|manifest| {
                                MatchedManifest::new(ctx, manifest, &pattern, self.mode, self.arch)
                            })
                            .filter(|matched_manifest| {
                                matched_manifest.should_match(self.installed)
                            })
                            .collect::<Vec<_>>();

                        if matches.is_empty() {
                            None
                        } else {
                            Some((bucket.name().to_string(), matches))
                        }
                    }
                    _ => None,
                },
            )
            .collect();

        if self.json {
            let json_matches: HashMap<String, Vec<MatchedOutput>> = buckets
                .into_iter()
                .map(|(bucket, matches)| {
                    let bucket_matches: Vec<MatchedOutput> = matches
                        .into_iter()
                        .map(MatchedManifest::into_output)
                        .collect();

                    (bucket, bucket_matches)
                })
                .collect();

            serde_json::to_writer_pretty(std::io::stdout(), &json_matches)?;
        } else {
            let mut matches: Sections<_> = buckets
                .into_iter()
                .map(|(bucket, matches)| {
                    let mut sections = vec![];

                    matches
                        .par_iter()
                        .map(MatchedManifest::to_section)
                        .collect_into_vec(&mut sections);

                    Section::new(Children::from(sections)).with_title(format!("'{bucket}' bucket:"))
                })
                .collect();

            matches.par_sort();

            print!("{matches}");
        }

        Ok(())
    }
}
