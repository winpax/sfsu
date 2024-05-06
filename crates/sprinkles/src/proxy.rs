use std::{num::ParseIntError, str::FromStr};

use crate::let_chain;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Proxy errors
pub enum Error {
    #[error("Invalid port: {0}")]
    InvalidPort(#[from] ParseIntError),
    #[error("Missing host")]
    MissingHost,
    #[error("Missing port")]
    MissingPort,
}

#[derive(Debug, Clone)]
/// A proxy struct
pub struct Proxy {
    username: Option<String>,
    password: Option<String>,
    host: String,
    port: u16,
}

impl Proxy {
    #[must_use]
    /// Construct a new proxy
    pub fn new(
        username: Option<String>,
        password: Option<String>,
        host: String,
        port: u16,
    ) -> Self {
        Self {
            username,
            password,
            host,
            port,
        }
    }
}

impl TryFrom<Proxy> for reqwest::Proxy {
    type Error = reqwest::Error;

    fn try_from(value: Proxy) -> Result<Self, Self::Error> {
        let proxy = reqwest::Proxy::all(format!("http://{}:{}", value.host, value.port))?;

        let proxy = let_chain!(let Some(username) = value.username; let Some(password) = value.password; {
            proxy.basic_auth(&username, &password)
        }; else proxy);

        Ok(proxy)
    }
}

impl FromStr for Proxy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (auth, host) = s
            .split_once('@')
            .map_or((None, s), |(auth, host)| (Some(auth), host));

        let (username, password) = let_chain! {
            let Some(auth) = auth; let Some(auth) = auth.split_once(':'); {
                (Some(auth.0.to_string()), Some(auth.1.to_string()))
            };
            else (None, None)
        };

        let mut parts = host.split(':');
        let host = parts.next().ok_or(Error::MissingHost)?.to_string();
        let port = parts.next().ok_or(Error::MissingPort)?.parse()?;

        Ok(Self::new(username, password, host, port))
    }
}

impl std::fmt::Display for Proxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(username) = &self.username {
            write!(f, "{username}:")?;
        }

        if let Some(password) = &self.password {
            write!(f, "{password}@")?;
        }

        write!(f, "{}", self.host)?;

        write!(f, ":{}", self.port)?;

        Ok(())
    }
}

mod ser_de {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};

    use super::Proxy;

    impl Serialize for Proxy {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_str(self)
        }
    }

    impl<'de> Deserialize<'de> for Proxy {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Proxy::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}
