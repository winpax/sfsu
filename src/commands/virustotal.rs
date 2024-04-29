use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use regex::Regex;
use sprinkles::{
    abandon,
    buckets::Bucket,
    calm_panic::CalmUnwrap,
    green,
    packages::SearchMode,
    progress::{style, ProgressOptions},
    red,
    requests::user_agent,
    yellow, Architecture, Scoop,
};

enum Status {
    Malicious,
    Suspicious,
    Undetected,
}

impl Status {
    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
    pub fn from_stats(dangerous: i64, total: i64) -> Self {
        let dangerous = dangerous as f64;
        let total = total as f64;
        let ratio = dangerous / total;

        if ratio > 0.1 {
            Self::Malicious
        } else if dangerous > 0.0 {
            Self::Suspicious
        } else {
            Self::Undetected
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    // TODO: Use manifest reference and -a flag for scanning installed apps
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

    #[clap(
        short,
        long,
        help = "Use the specified architecture, if the app supports it",
        default_value_t = Architecture::ARCH
    )]
    arch: Architecture,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let config = Scoop::config()?;
        let api_key = config
            .virustotal_api_key
            .unwrap_or_else(|| abandon!("No virustotal api key found"));

        let client = vt3::VtClient::new(&api_key).user_agent(&user_agent());

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

        let manifests = matching_buckets
            .into_par_iter()
            .flat_map(
                |bucket| match bucket.matches(false, &pattern, SearchMode::Name) {
                    Ok(manifests) => manifests,
                    _ => vec![],
                },
            )
            .collect::<Vec<_>>();

        let matches = manifests
            .into_par_iter()
            .progress_with_style(style(Some(ProgressOptions::PosLen), None))
            .map(|manifest| {
                let hash = manifest.install_config(self.arch).hash;

                if let Some(hash) = hash {
                    if let Ok(file_info) = client.clone().file_info(&hash.to_string()) {
                        return anyhow::Ok(Some((manifest, file_info)));
                    } else {
                        return anyhow::Ok(None);
                    }
                }

                anyhow::Ok(None)
            })
            .filter_map(Result::transpose)
            .collect::<Result<Vec<_>, _>>()?;

        for (manifest, file_info) in matches {
            let Some(stats) = file_info
                .data
                .and_then(|data| data.attributes)
                .and_then(|attributes| attributes.last_analysis_stats)
            else {
                red!("No data found for {}", manifest.name);
                continue;
            };

            let detected = stats
                .malicious
                .map(|m| m + stats.suspicious.unwrap_or_default())
                .unwrap_or_default();
            let total = detected + stats.undetected.unwrap_or_default();

            let file_status = Status::from_stats(detected, total);

            let file_url = format!(
                "https://www.virustotal.com/gui/url/{hash}",
                hash = manifest.install_config(self.arch).hash.unwrap()
            );

            let info = format!(
                "{}/{}: {detected}/{total}. See more at {file_url}",
                manifest.bucket, manifest.name,
            );

            match file_status {
                Status::Malicious => red!("{info}"),
                Status::Suspicious => yellow!("{info}"),
                Status::Undetected => green!("{info}"),
            }
        }

        Ok(())
    }
}
