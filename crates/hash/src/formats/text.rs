use std::collections::HashMap;

use regex::Regex;
use strum::{Display, EnumIter};

use crate::ops::{Substitute, SubstituteBuilder};

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
            let regex: &'static str = field.into();

            map.insert(field.to_string(), regex.to_string());
        }

        map
    }
}

pub fn parse_text(
    source: impl AsRef<str>,
    file_names: &[impl AsRef<str>],
    substitutions: HashMap<String, String>,
    regex: String,
) -> Vec<(String, String)> {
    let regex = if regex.is_empty() {
        r"^\s*([a-fA-F0-9]+)\s*$".to_string()
    } else {
        regex
    };

    let substituted = {
        let mut regex = regex;

        // Substitute regex templates for finding hashes
        regex.substitute(RegexTemplates::into_substitute_map(), false);
        // Substitute provided substitutions (i.e url, basename, etc.)
        regex.substitute(substitutions, true);

        regex
    };

    todo!()
}
