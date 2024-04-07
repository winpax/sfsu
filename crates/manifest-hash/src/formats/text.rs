use std::collections::HashMap;

use itertools::Itertools as _;
use regex::Regex;
use strum::{Display, EnumIter};

use crate::ops::Substitute;

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Base64 decoding: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

// Convert from https://github.com/ScoopInstaller/Scoop/blob/f93028001fbe5c78cc41f59e3814d2ac8e595724/lib/autoupdate.ps1#L75

#[derive(Debug, Copy, Clone, Display, EnumIter)]
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
    fn into_substitute_map() -> HashMap<String, String> {
        use strum::IntoEnumIterator;

        let mut map = HashMap::new();

        for field in Self::iter() {
            let field_name = format!("${field}");
            let regex: &'static str = field.into();

            map.insert(field_name, regex.to_string());
        }

        map
    }
}

pub fn parse_text(
    source: impl AsRef<str>,
    substitutions: HashMap<String, String>,
    regex: String,
) -> Result<Option<String>, TextError> {
    // TODO: Incorporate file_names

    let regex = if regex.is_empty() {
        r"^\s*([a-fA-F0-9]+)\s*$".to_string()
    } else {
        regex
    };

    let substituted = {
        let mut regex = regex;

        // Substitute regex templates for finding hashes
        regex.substitute(&RegexTemplates::into_substitute_map(), false);
        // Substitute provided substitutions (i.e url, basename, etc.)
        regex.substitute(&substitutions, true);

        debug!("{regex}");

        Regex::new(&regex)?
    };

    let hashes = substituted
        .find_iter(source.as_ref())
        .map(|hash| hash.as_str().replace(' ', ""))
        .collect_vec();

    eprintln!("Hashes length after subbing searching: {}", hashes.len());

    // Convert base64 encoded hashes
    let hash = if let Some(hash) = hashes.first() {
        let base64_regex = Regex::new(
            r"^(?:[A-Za-z0-9+\/]{4})*(?:[A-Za-z0-9+\/]{2}==|[A-Za-z0-9+\/]{3}=|[A-Za-z0-9+\/]{4})$",
        )
        .expect("valid base64 regex");

        if let Some(base64_hash) = base64_regex.find(hash) {
            let invalid_base64 =
                Regex::new(r"^[a-fA-F0-9]+$").expect("valid \"invalid base64\" regex");

            let base64_hash = base64_hash.as_str();

            // Detects an invalid base64 string
            if !(invalid_base64.is_match(base64_hash)
                && [32, 40, 64, 128].contains(&base64_hash.len()))
            {
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

                Some(decoded_hash)
            } else {
                Some(hash.clone())
            }
        } else {
            Some(hash.clone())
        }
    } else {
        println!("Didn't find first regex");
        let filename_regex = {
            let regex = r"([a-fA-F0-9]{32,128})[\x20\t]+.*`$basename(?:[\x20\t]+\d+)?"
                .to_string()
                .into_substituted(&substitutions, true);

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
    use std::collections::HashMap;

    use url::Url;

    use super::*;

    use crate::requests::BlockingClient;

    #[test]
    fn test_finding_pixelflasher_hashes() {
        let  text_url: String = "https://github.com/badabing2005/PixelFlasher/releases/download/v6.9.1.0/PixelFlasher.exe.sha256".to_string();
        const FIND_REGEX: &str = "$sha256";

        let url = Url::parse(&text_url).unwrap();

        let mut subs = HashMap::new();

        let no_fragment = if let Some(fragment) = url.fragment() {
            text_url.replace(&format!("#{}", fragment), "")
        } else {
            text_url.clone()
        };

        subs.insert("$url".to_string(), no_fragment.clone());
        subs.insert("$baseurl".to_string(), no_fragment);

        let text_file: String = BlockingClient::new()
            .get(text_url)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let hash = parse_text(text_file, subs, FIND_REGEX.to_string())
            .unwrap()
            .expect("found hash");

        assert_eq!(
            "8a0d9ab83478a6389d6ac0a6294136f9e81b8f5a9c312cfc7a855ef9f9a2f0da",
            hash
        );
    }

    #[test]
    fn test_finding_mysql_hashes() {
        let mut text_url: String = "https://dev.mysql.com/downloads/mysql/".to_string();
        const FIND_REGEX: &str = "md5\">$md5";

        let url = Url::parse(&text_url).unwrap();

        if let Some(fragment) = url.fragment() {
            text_url = text_url.replace(&format!("#{}", fragment), "");
        }

        let mut subs = HashMap::new();

        let no_fragment = if let Some(fragment) = url.fragment() {
            text_url.replace(&format!("#{}", fragment), "")
        } else {
            text_url.clone()
        };

        subs.insert("$url".to_string(), no_fragment.clone());
        subs.insert("$baseurl".to_string(), no_fragment);

        let response = BlockingClient::new().get(text_url).send().unwrap();
        let text_file = response.text().unwrap();

        std::fs::write("pp.html", &text_file).unwrap();

        let hash = parse_text(text_file, subs, FIND_REGEX.to_string())
            .unwrap()
            .expect("found hash");

        assert_eq!("186efc230e44ded93b5aa89193a6fcbf", hash);
    }
}
