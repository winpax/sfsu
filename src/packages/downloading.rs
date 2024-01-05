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
}
