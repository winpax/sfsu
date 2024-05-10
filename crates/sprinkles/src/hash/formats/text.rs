use itertools::Itertools as _;
use quork::traits::list::ListVariants;
use regex::Regex;
use strum::Display;

use crate::hash::substitutions::{Substitute, SubstitutionMap};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Base64 decoding: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

// Convert from https://github.com/ScoopInstaller/Scoop/blob/f93028001fbe5c78cc41f59e3814d2ac8e595724/lib/autoupdate.ps1#L75

#[derive(Debug, Copy, Clone, Display, ListVariants)]
#[strum(serialize_all = "lowercase")]
enum RegexTemplates {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Checksum,
    Base64,
}

impl From<RegexTemplates> for &'static str {
    fn from(value: RegexTemplates) -> Self {
        match value {
            RegexTemplates::Md5 => r"([a-fA-F0-9]{32})",
            RegexTemplates::Sha1 => r"([a-fA-F0-9]{40})",
            RegexTemplates::Sha256 => r"([a-fA-F0-9]{64})",
            RegexTemplates::Sha512 => r"([a-fA-F0-9]{128})",
            RegexTemplates::Checksum => r"([a-fA-F0-9]{32,128})",
            RegexTemplates::Base64 => r"([a-zA-Z0-9+\/=]{24,88})",
        }
    }
}

impl RegexTemplates {
    fn into_substitute_map() -> SubstitutionMap {
        let mut map = SubstitutionMap::new();

        for field in Self::VARIANTS {
            let field_name = format!("${field}");
            let regex: &'static str = field.into();

            map.insert(field_name, regex.to_string());
        }

        map
    }
}

pub fn parse_text(
    source: impl AsRef<str>,
    substitutions: &SubstitutionMap,
    regex: impl AsRef<str>,
) -> Result<Option<String>, Error> {
    // TODO: Incorporate file_names

    debug!("Parsing from regex: {}", regex.as_ref());

    let regex = if regex.as_ref().is_empty() {
        r"^\s*([a-fA-F0-9]+)\s*$".to_string()
    } else {
        regex.as_ref().to_string()
    };

    let substituted = {
        let mut regex = regex;

        // Substitute regex templates for finding hashes
        regex.substitute(&RegexTemplates::into_substitute_map(), false);
        // Substitute provided substitutions (i.e url, basename, etc.)
        regex.substitute(substitutions, true);

        debug!("{regex}");

        Regex::new(&regex)?
    };

    debug!("Source: {}", source.as_ref());

    let hash = substituted
        .captures(source.as_ref())
        .and_then(|captures| {
            // Get the first capture group (i.e the actual hash value)
            captures.get(1)
        })
        .map(|hash| hash.as_str().replace(' ', ""));

    // Convert base64 encoded hashes
    let hash = if let Some(hash) = hash {
        debug!("Found hash: {hash}");

        let base64_regex = Regex::new(
            r"^(?:[A-Za-z0-9+\/]{4})*(?:[A-Za-z0-9+\/]{2}==|[A-Za-z0-9+\/]{3}=|[A-Za-z0-9+\/]{4})$",
        )
        .expect("valid base64 regex");

        base64_regex
            .find(&hash)
            .and_then(|base64_hash| {
                debug!("Found base64 hash");
                let invalid_base64 =
                    Regex::new(r"^[a-fA-F0-9]+$").expect("valid \"invalid base64\" regex");

                let base64_hash = base64_hash.as_str();

                // Detects an invalid base64 string
                (!(invalid_base64.is_match(base64_hash)
                    && [32, 40, 64, 128].contains(&base64_hash.len())))
                .then(|| {
                    use base64::prelude::*;

                    let decoded_hash =
                        if let Ok(decoded) = BASE64_STANDARD.decode(base64_hash.as_bytes()) {
                            let mut decoded_hash = String::new();

                            decoded
                                .into_iter()
                                .for_each(|byte| decoded_hash += &format!("{byte:x}"));

                            decoded_hash
                        } else {
                            hash.clone()
                        };

                    decoded_hash
                })
            })
            .or_else(|| Some(hash.clone()))
    } else {
        println!("Didn't find first regex");
        let filename_regex = {
            let regex = r"([a-fA-F0-9]{32,128})[\x20\t]+.*`$basename(?:[\x20\t]+\d+)?"
                .to_string()
                .into_substituted(substitutions, true);

            Regex::new(&regex)?
        };

        let mut temp_hash = filename_regex
            .find_iter(source.as_ref())
            .map(|hash| hash.as_str().to_string())
            .collect_vec()
            .first()
            .cloned();

        if temp_hash.is_none() {
            let metalink_regex = Regex::new(r"<hash[^>]+>([a-fA-F0-9]{64})")?;

            temp_hash = metalink_regex
                .find_iter(source.as_ref())
                .map(|hash| hash.as_str().to_string())
                .collect_vec()
                .first()
                .cloned();
        }

        temp_hash
    };

    Ok(hash.map(|hash| hash.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    use crate::{
        buckets::Bucket, contexts::User,
        packages::models::manifest::HashExtractionOrArrayOfHashExtractions, requests::Client,
    };

    #[test]
    #[ignore = "replaced by testhandler tests"]
    fn test_finding_vcredistaio_hashes() {
        let ctx = User::new();

        let manifest = Bucket::from_name(&ctx, "extras")
            .unwrap()
            .get_manifest("vcredist-aio")
            .unwrap();

        let (text_url, regex) =
            if let HashExtractionOrArrayOfHashExtractions::HashExtraction(extraction) =
                manifest.autoupdate.unwrap().default_config.hash.unwrap()
            {
                (
                    extraction
                        .url
                        .unwrap()
                        .replace("$version", manifest.version.as_str()),
                    extraction.regex.unwrap(),
                )
            } else {
                panic!("No hash extraction found");
            };

        let text_file: String = Client::blocking()
            .get(text_url)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let mut substitutions = SubstitutionMap::new();

        substitutions.insert(
            "$basename".into(),
            "VisualCppRedist_AIO_x86_x64_81.zip".into(),
        );

        let hash = parse_text(text_file, &substitutions, regex)
            .unwrap()
            .expect("found hash");

        let actual_hash = manifest.install_config.hash.unwrap().to_string();

        assert_eq!(actual_hash, hash);
    }

    #[ignore = "Finds the first hash on the page, which is not the correct hash. Note that this is the same way that Scoop does it, so I'm not sure how it figures out the correct hash in the manifest."]
    #[test]
    fn test_finding_mysql_hashes() {
        const FIND_REGEX: &str = "md5\">$md5";

        let ctx = User::new();
        let mut text_url: String = "https://dev.mysql.com/downloads/mysql/".to_string();

        let url = Url::parse(&text_url).unwrap();

        if let Some(fragment) = url.fragment() {
            text_url = text_url.replace(&format!("#{fragment}"), "");
        }

        let mut substitutions = SubstitutionMap::new();

        let no_fragment = if let Some(fragment) = url.fragment() {
            text_url.replace(&format!("#{fragment}"), "")
        } else {
            text_url.clone()
        };

        substitutions.insert("$url".to_string(), no_fragment.clone());
        substitutions.insert("$baseurl".to_string(), no_fragment);

        let response = Client::blocking().get(text_url).send().unwrap();
        let text_file = response.text().unwrap();

        let hash = "md5:".to_string()
            + &parse_text(text_file, &substitutions, FIND_REGEX)
                .unwrap()
                .expect("found hash");

        let actual_hash = {
            Bucket::from_name(&ctx, "main")
                .unwrap()
                .get_manifest("mysql")
                .unwrap()
                .architecture
                .unwrap()
                .x64
                .unwrap()
                .hash
                .unwrap()
                .to_string()
        };

        assert_eq!(actual_hash, hash);
    }
}
