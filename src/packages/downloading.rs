use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadUrl {
    pub url: String,
    pub file_name: Option<String>,
}

impl DownloadUrl {
    #[must_use]
    pub fn new(url: String, file_name: Option<String>) -> Self {
        Self { url, file_name }
    }

    #[must_use]
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

    pub fn into_cache_path(&self) -> PathBuf {
        self.into()
    }
}

impl From<DownloadUrl> for PathBuf {
    fn from(url: DownloadUrl) -> Self {
        url.into()
    }
}

impl From<&DownloadUrl> for PathBuf {
    fn from(url: &DownloadUrl) -> Self {
        const INVALID_FILE_NAME_CHARS: &[char] = &[
            '#', '"', '<', '>', '|', '\0', 1 as char, 2 as char, 3 as char, 4 as char, 5 as char,
            6 as char, 7 as char, 8 as char, 9 as char, 10 as char, 11 as char, 12 as char,
            13 as char, 14 as char, 15 as char, 16 as char, 17 as char, 18 as char, 19 as char,
            20 as char, 21 as char, 22 as char, 23 as char, 24 as char, 25 as char, 26 as char,
            27 as char, 28 as char, 29 as char, 30 as char, 31 as char, ':', '*', '?', '\\', '/',
        ];

        let safe_url = url
            .url
            .chars()
            .map(|char| {
                if INVALID_FILE_NAME_CHARS.contains(&char) {
                    '_'
                } else {
                    char
                }
            })
            .collect::<String>();

        PathBuf::from(safe_url)
    }
}
