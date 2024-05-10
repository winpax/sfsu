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

    #[allow(dead_code)]
    fn substitute(&mut self, submap: &SubstitutionMap);

    fn submap(&self) -> SubstitutionMap;
}

impl UrlExt for Url {
    fn remote_filename(&self) -> String {
        let leaf = self.leaf().expect("url leaf");

        let query_regex = Regex::new(r".*[?=]+([\w._-]+)").expect("valid query regex");
        let version_regex = Regex::new(r"^[v.\d]+$").expect("valid version regex");

        if let Some(query_filename) = query_regex
            .captures(&leaf)
            .and_then(|captures| captures.get(1).map(|capture| capture.as_str().to_string()))
        {
            return query_filename;
        }

        if !leaf.contains('.') || version_regex.is_match(&leaf) {
            return leaf;
        }

        if !leaf.contains('.') {
            if let Some(fragment) = self.fragment() {
                return fragment.trim_matches('#').trim_matches('/').to_string();
            }
        }

        leaf
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

    fn submap(&self) -> SubstitutionMap {
        let stripped_url = {
            let mut url = self.clone();
            url.strip_fragment();
            url
        };

        let basename = urlencoding::decode(&self.remote_filename())
            .expect("UTF-8")
            .to_string();

        let mut map = SubstitutionMap::new();

        map.insert("$url".into(), stripped_url.to_string());
        map.insert("$baseurl".into(), {
            let mut base_url = stripped_url.clone();
            base_url.strip_filename();
            base_url.to_string()
        });
        map.insert("$basenameNoExt".into(), strip_ext(&basename).to_string());
        map.insert("$basename".into(), basename);

        if let Some(url_no_ext) = stripped_url.leaf().as_ref().map(|fname| strip_ext(fname)) {
            map.insert("$urlNoExt".into(), url_no_ext.to_string());
        }

        map
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
    fn test_basename() {
        let  url = Url::parse("https://github.com/abbodi1406/vcredist/releases/download/v0.80.0/VisualCppRedist_AIO_x86_x64_80.zip").unwrap();
        let basename = url.remote_filename();
        assert_eq!(basename.as_str(), "VisualCppRedist_AIO_x86_x64_80.zip");
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
