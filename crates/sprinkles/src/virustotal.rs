pub mod models;

use reqwest::StatusCode;

use crate::requests::AsyncClient;

use self::models::scan::FileData;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// `VirusTotal` API Errors
pub enum Error {
    #[error("HTTP error: {0}")]
    HTTP(#[from] reqwest::Error),
    #[error("Status code: {0}, message: {1}")]
    StatusError(StatusCode, String),
}
/// `VirusTotal` API Result
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct VtClient {
    api_key: String,
    endpoint: String,
}

impl VtClient {
    #[must_use]
    /// Create a new `VirusTotal` Client
    pub fn new(api_key: &str) -> Self {
        VtClient {
            api_key: api_key.into(),
            endpoint: "https://www.virustotal.com/api/v3".into(),
        }
    }

    /// Retrieve `public_api.file` scan reports
    /// id: SHA-256, SHA-1 or MD5 identifying the `public_api.file`
    pub async fn file_info(&self, id: &str) -> Result<FileData> {
        let url = format!("{}/files/{}", &self.endpoint, id);

        let client = AsyncClient::new();
        let resp = client
            .get(url)
            .header("Accept", "application/json")
            .header("x-apikey", &self.api_key)
            .send()
            .await?;

        let status = resp.status();

        match status {
            StatusCode::OK => Ok(resp.json().await?), // 200
            _ => Err(Error::StatusError(status, resp.text().await?)),
        }
    }
}
