#![doc(hidden)]

//! HTTP request helpers and defaults
//!
//! Note that this is primarily intended for internal SFSU use, and is not recommended for general use.
//! You are welcome to do so, but be aware that the API may change without warning, and it will likely not meet your requirements.

use derive_more::Deref;
use reqwest::header::HeaderMap;

#[must_use]
/// Get user agent for sfsu
pub fn user_agent() -> String {
    use std::env::consts::{ARCH, OS};

    const VERSION: &str = env!("CARGO_PKG_VERSION");

    format!("Scoop/1.0 (+https://scoop.sh/) sfsu/{VERSION} ({ARCH}) ({OS})")
}

#[must_use]
/// Construct default headers for requests
///
/// # Panics
/// - Invalid headers
pub fn default_headers() -> HeaderMap {
    use reqwest::header::{HeaderValue, ACCEPT, USER_AGENT};

    let mut headers = HeaderMap::new();

    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(USER_AGENT, user_agent().parse().unwrap());

    headers
}

#[derive(Debug, Clone, Deref)]
/// A blocking client with sane defaults for SFSU
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
/// An async client with sane defaults for SFSU
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
