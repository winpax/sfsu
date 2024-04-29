use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use sprinkles::{abandon, buckets::Bucket, calm_panic::CalmUnwrap, packages::SearchMode, Scoop};

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

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let config = Scoop::config()?;
        let api_key = config
            .virustotal_api_key
            .unwrap_or_else(|| abandon!("No virustotal api key found"));

        let (bucket, raw_pattern) =
            if let Some((bucket, raw_pattern)) = self.pattern.split_once('/') {
                // Bucket flag overrides bucket/package syntax
                (
                    Some(self.bucket.unwrap_or(bucket.to_string())),
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

        let matching_buckets: Vec<Bucket> = if let Some(Ok(bucket)) = bucket.map(Bucket::from_name)
        {
            vec![bucket]
        } else {
            Bucket::list_all()?
        };

        let mut matches: Vec<_> = matching_buckets
            .par_iter()
            .flat_map(
                |bucket| match bucket.matches(false, &pattern, SearchMode::Name) {
                    Ok(manifests) => manifests,
                    _ => vec![],
                },
            )
            .collect();

        todo!()
    }
}
