use std::borrow::Cow;

use regex::Regex;
use url::Url;

pub fn strip_ext(file_name: &str) -> Cow<'_, str> {
    let ext_regex = Regex::new(r"\.[^\.]*$").expect("valid extension regex");

    ext_regex.replace_all(file_name, "")
}

pub fn remote_filename(url: &Url) -> String {
    let basename = {
        let basename = url.path();
        url.query().map_or_else(
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
    } else if let Some(leaf) = leaf(url)
        && (!basename.contains('.') || version_regex.is_match(&basename))
    {
        leaf
    } else if let Some(fragment) = url.fragment()
        && !basename.contains('.')
    {
        fragment.trim_matches('#').trim_matches('/').to_string()
    } else {
        basename
    }
}

pub fn strip_fragment(url: &mut Url) {
    url.set_fragment(None);
}

pub fn strip_filename(url: &mut Url) {
    strip_fragment(url);

    _ = url.path_segments_mut().map(|mut segments| {
        // Remove filename
        segments.pop();
        // Remove trailing slash
        segments.pop_if_empty();
    });
}

pub fn leaf(url: &Url) -> Option<String> {
    url.path_segments()
        .and_then(|segments| segments.last().map(ToString::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_fragment() {
        let mut url = Url::parse("https://example.com/#fragment").unwrap();
        strip_fragment(&mut url);
        assert_eq!(url.as_str(), "https://example.com/");
    }

    #[test]
    fn test_strip_filename() {
        let mut url = Url::parse("https://github.com/abbodi1406/vcredist/releases/download/v0.80.0/VisualCppRedist_AIO_x86_x64_80.zip").unwrap();
        strip_filename(&mut url);
        assert_eq!(
            url.as_str(),
            "https://github.com/abbodi1406/vcredist/releases/download/v0.80.0"
        );
    }
}
