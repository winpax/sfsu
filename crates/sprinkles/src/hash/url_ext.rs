use std::borrow::Cow;

use regex::Regex;
use url::Url;

use super::substitutions::{Substitute, SubstitutionMap};

pub fn strip_ext(file_name: &str) -> Cow<'_, str> {
    let ext_regex = Regex::new(r"\.[^\.]*$").expect("valid extension regex");

    ext_regex.replace_all(file_name, "")
}

pub trait UrlExt {
    fn remote_filename(&self) -> String;

    fn strip_fragment(&mut self);

    fn strip_filename(&mut self);

    fn leaf(&self) -> Option<String>;

    fn substitute(&mut self, submap: &SubstitutionMap);
}

impl UrlExt for Url {
    fn remote_filename(&self) -> String {
        let basename = {
            let basename = self.path();
            self.query().map_or_else(
                || basename.to_string(),
                |query| format!("{basename}?{query}"),
            )
        };

        let query_regex = Regex::new(r".*[?=]+([\w._-]+)").expect("valid query regex");
        let version_regex = Regex::new(r"^[v.\d]+$").expect("valid version regex");

        if let Some(query_filename) = query_regex
            .captures(&basename)
            .and_then(|captures| captures.get(1).map(|capture| capture.as_str().to_string()))
        {
            query_filename
        } else if let Some(leaf) = self.leaf()
            && (!basename.contains('.') || version_regex.is_match(&basename))
        {
            leaf
        } else if let Some(fragment) = self.fragment()
            && !basename.contains('.')
        {
            fragment.trim_matches('#').trim_matches('/').to_string()
        } else {
            basename
        }
    }

    fn strip_fragment(&mut self) {
        self.set_fragment(None);
    }

    fn strip_filename(&mut self) {
        self.strip_fragment();

        _ = self.path_segments_mut().map(|mut segments| {
            // Remove filename
            segments.pop();
            // Remove trailing slash
            segments.pop_if_empty();
        });
    }

    fn leaf(&self) -> Option<String> {
        self.path_segments()
            .and_then(|segments| segments.last().map(ToString::to_string))
    }

    fn substitute(&mut self, submap: &SubstitutionMap) {
        let mut url = self.to_string();

        url.substitute(submap, false);

        *self = Url::parse(&url).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_fragment() {
        let mut url = Url::parse("https://example.com/#fragment").unwrap();
        url.strip_fragment();
        assert_eq!(url.as_str(), "https://example.com/");
    }

    #[test]
    fn test_strip_filename() {
        let mut url = Url::parse("https://github.com/abbodi1406/vcredist/releases/download/v0.80.0/VisualCppRedist_AIO_x86_x64_80.zip").unwrap();
        url.strip_filename();
        assert_eq!(
            url.as_str(),
            "https://github.com/abbodi1406/vcredist/releases/download/v0.80.0"
        );
    }
}
