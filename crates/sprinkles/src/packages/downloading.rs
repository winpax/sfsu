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

    #[must_use]
    /// Convert into a path buf
    ///
    /// # Panics
    /// - If the hardcoded regex is invalid
    pub fn to_path_buf(&self) -> PathBuf {
        let cache_path_regex = Regex::new(r"[^\w\.\-]+").expect("valid regex");

        let safe_url = cache_path_regex.replace_all(&self.url, "_");

        let file_name = PathBuf::from(safe_url.to_string());

        if let Some(frag) = &self.file_name {
            let orig_ext = file_name.extension();
            let mut file_name = file_name.display().to_string();
            file_name += "_";
            file_name += frag;

            let file_name = PathBuf::from(file_name);

            if let Some(ext) = orig_ext {
                file_name.with_extension(ext)
            } else {
                file_name
            }
        } else {
            file_name
        }
    }
}

impl From<&DownloadUrl> for PathBuf {
    fn from(url: &DownloadUrl) -> Self {
        url.to_path_buf()
    }
}
