#![doc(hidden)]

//! HTTP request helpers and defaults
//!
//! Note that this is primarily intended for internal SFSU use, and is not recommended for general use.
//! You are welcome to do so, but be aware that the API may change without warning, and it will likely not meet your requirements.

use derive_more::Deref;
use reqwest::header::HeaderMap;

use crate::config;

#[must_use]
#[deprecated(note = "Use `USER_AGENT` instead")]
#[cfg(not(feature = "v2"))]
/// Get user agent for sfsu
pub const fn user_agent<'a>() -> &'a str {
    USER_AGENT
}

/// User agent for sfsu
pub const USER_AGENT: &str = {
    use const_format::formatcp;
    use std::env::consts::{ARCH, OS};

    formatcp!(
        "Scoop/1.0 (+https://scoop.sh/) sfsu/{} ({}) ({})",
        env!("CARGO_PKG_VERSION"),
        ARCH,
        OS
    )
};

#[must_use]
/// Construct default headers for requests
///
/// # Panics
/// - Invalid headers
pub fn default_headers() -> HeaderMap {
    use reqwest::header::{self, HeaderValue};

    let mut headers = HeaderMap::new();

    headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(header::USER_AGENT, USER_AGENT.parse().unwrap());

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
        let client = reqwest::blocking::Client::builder().default_headers(default_headers());

        let client = if let Some(proxy) = config::Scoop::load().expect("scoop config").proxy {
            client.proxy(proxy.try_into().expect("valid reqwest proxy"))
        } else {
            client
        };

        Self(client.build().unwrap())
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
        let client = reqwest::Client::builder().default_headers(default_headers());

        let client = if let Some(proxy) = config::Scoop::load().expect("scoop config").proxy {
            client.proxy(proxy.try_into().expect("valid reqwest proxy"))
        } else {
            client
        };

        Self(client.build().unwrap())
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
