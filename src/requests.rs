use derive_more::Deref;
use reqwest::header::HeaderMap;

pub const USER_AGENT: &str = {
    use const_format::formatcp;
    use std::env::consts::{ARCH, OS};

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    formatcp!(
        "Scoop/1.0 (+https://scoop.sh/) sfsu/{} ({}) ({})",
        VERSION,
        ARCH,
        OS,
    )
};

#[must_use]
/// Construct default headers for requests
///
/// # Panics
/// - Invalid headers
pub fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.append("User-Agent", USER_AGENT.parse().unwrap());

    headers
}

#[derive(Debug, Clone, Deref)]
pub struct BlockingClient(reqwest::blocking::Client);

impl BlockingClient {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for BlockingClient {
    fn default() -> Self {
        Self(
            reqwest::blocking::Client::builder()
                .default_headers(default_headers())
                .build()
                .unwrap(),
        )
    }
}

#[derive(Debug, Clone, Deref)]
pub struct AsyncClient(reqwest::Client);

impl AsyncClient {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for AsyncClient {
    fn default() -> Self {
        Self(
            reqwest::Client::builder()
                .default_headers(default_headers())
                .build()
                .unwrap(),
        )
    }
}
