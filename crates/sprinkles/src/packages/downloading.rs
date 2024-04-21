//! Downloading helpers for packages

use std::path::PathBuf;

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A URL to download
pub struct DownloadUrl {
    /// The URL
    pub url: String,
    /// The destination file name
    pub file_name: Option<String>,
}

impl DownloadUrl {
    #[must_use]
    /// Create a new download URL
    pub fn new(url: String, file_name: Option<String>) -> Self {
        Self { url, file_name }
    }

    #[must_use]
    /// Create a new download URL from a string
    pub fn from_string(url: String) -> Self {
        if let Some((url, file_name)) = url.split_once("#/") {
            Self {
                url: url.to_string(),
                file_name: Some(file_name.to_string()),
            }
        } else {
            Self {
                url,
                file_name: None,
            }
        }
    }

    #[must_use]
    /// Get the cache path for the download URL
    pub fn into_cache_path(&self) -> PathBuf {
        self.into()
    }
}

impl From<&DownloadUrl> for PathBuf {
    fn from(url: &DownloadUrl) -> Self {
        let cache_path_regex = Regex::new(r"[^\w\.\-]+").expect("valid regex");

        let safe_url = cache_path_regex.replace_all(&url.url, "_");

        PathBuf::from(safe_url.to_string())
    }
}
