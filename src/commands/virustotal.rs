use clap::Parser;
use rayon::prelude::*;
use regex::Regex;
use sprinkles::{
    abandon, buckets::Bucket, calm_panic::CalmUnwrap, green, packages::SearchMode, red,
    requests::user_agent, yellow, Architecture, Scoop,
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
        // TODO: Implement rate limits

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

        let matches = matching_buckets
            .into_par_iter()
            .flat_map(
                |bucket| match bucket.matches(false, &pattern, SearchMode::Name) {
                    Ok(manifests) => manifests,
                    _ => vec![],
                },
            )
            .map(|manifest| {
                let hash = manifest.install_config(self.arch).hash;

                if let Some(hash) = hash {
                    let file_info = client.clone().file_info(&hash)?;

                    return anyhow::Ok(Some((manifest, file_info)));
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

            match file_status {
                Status::Malicious => red!(
                    "{}/{}: {}/{}",
                    manifest.bucket,
                    manifest.name,
                    detected,
                    total
                ),
                Status::Suspicious => yellow!(
                    "{}/{}: {}/{}",
                    manifest.bucket,
                    manifest.name,
                    detected,
                    total
                ),
                Status::Undetected => green!(
                    "{}/{}: {}/{}",
                    manifest.bucket,
                    manifest.name,
                    detected,
                    total
                ),
            }
        }

        Ok(())
    }
}
