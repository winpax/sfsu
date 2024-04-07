use derive_more::Deref;
use reqwest::header::HeaderMap;

#[must_use]
/// Construct default headers for requests
///
/// # Panics
/// - Invalid headers
pub fn default_headers() -> HeaderMap {
    use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        USER_AGENT,
        {
            use std::env::consts::{ARCH, OS};

            const VERSION: &str = env!("CARGO_PKG_VERSION");

            format!("Scoop/1.0 (+https://scoop.sh/) sfsu/{VERSION} ({ARCH}) ({OS})")
        }
        .parse()
        .unwrap(),
    );

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
