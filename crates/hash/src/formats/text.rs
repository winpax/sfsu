use regex::Regex;

enum RegexTemplates {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Checksum,
    Base64,
}

impl From<RegexTemplates> for Regex {
    fn from(value: RegexTemplates) -> Self {
        match value {
            RegexTemplates::Md5 => Regex::new(r"([a-fA-F0-9]{32})"),
            RegexTemplates::Sha1 => Regex::new(r"([a-fA-F0-9]{40})"),
            RegexTemplates::Sha256 => Regex::new(r"([a-fA-F0-9]{64})"),
            RegexTemplates::Sha512 => Regex::new(r"([a-fA-F0-9]{128})"),
            RegexTemplates::Checksum => Regex::new(r"([a-fA-F0-9]{32,128})"),
            RegexTemplates::Base64 => Regex::new(r"([a-zA-Z0-9+\/=]{24,88})"),
        }
        .unwrap()
    }
}

pub fn parse_text(
    source: impl AsRef<str>,
    file_names: &[impl AsRef<str>],
    regex: Regex,
) -> Vec<(String, String)> {
    todo!()
}
