#![doc(hidden)]

//! HTTP request helpers and defaults
//!
//! Note that this is primarily intended for internal SFSU use, and is not recommended for general use.
//! You are welcome to do so, but be aware that the API may change without warning, and it will likely not meet your requirements.

use derive_more::Deref;
use reqwest::header::HeaderMap;

#[must_use]
/// Get user agent for sfsu
pub const fn user_agent<'a>() -> &'a str {
    // A little type hack to allow a string reference in inline_const
    type Stref = &'static str;

    crate::inline_const!(
        Stref
        {
            use const_format::formatcp;
            use std::env::consts::{ARCH, OS};
            const VERSION: &str = env!("CARGO_PKG_VERSION");

            formatcp!(
                "Scoop/1.0 (+https://scoop.sh/) sfsu/{} ({}) ({})",
                VERSION,
                ARCH,
                OS
            )
        }
    )
}

#[must_use]
/// Construct default headers for requests
///
/// # Panics
/// - Invalid headers
pub fn default_headers() -> HeaderMap {
    use reqwest::header::{self, HeaderValue};

    let mut headers = HeaderMap::new();

    headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(header::USER_AGENT, user_agent().parse().unwrap());

    headers
}

pub trait ClientLike<T>
where
    Self: Default,
{
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    fn client(&self) -> &T;
}

pub struct Client;

impl Client {
    #[must_use]
    pub fn asynchronous() -> AsyncClient {
        Self::create()
    }

    #[must_use]
    pub fn blocking() -> BlockingClient {
        Self::create()
    }

    #[must_use]
    pub fn create<T, C: ClientLike<T>>() -> C {
        C::new()
    }
}

#[derive(Debug, Clone, Deref)]
/// A blocking client with sane defaults for SFSU
pub struct BlockingClient(reqwest::blocking::Client);

impl ClientLike<reqwest::blocking::Client> for BlockingClient {
    fn client(&self) -> &reqwest::blocking::Client {
        &self.0
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

impl ClientLike<reqwest::Client> for AsyncClient {
    fn client(&self) -> &reqwest::Client {
        &self.0
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

impl ClientLike<reqwest::Client> for reqwest::Client {
    fn new() -> Self {
        AsyncClient::new().0
    }

    fn client(&self) -> &reqwest::Client {
        self
    }
}

impl ClientLike<reqwest::blocking::Client> for reqwest::blocking::Client {
    fn new() -> Self {
        BlockingClient::new().0
    }

    fn client(&self) -> &reqwest::blocking::Client {
        self
    }
}
