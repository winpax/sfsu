//! Proxy helpers

use std::{
    net::{AddrParseError, SocketAddr},
    num::ParseIntError,
    str::FromStr,
};

use crate::let_chain;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Proxy errors
pub enum Error {
    #[error("Invalid port: {0}")]
    InvalidPort(#[from] ParseIntError),
    #[error("Loading system proxy from registry: {0}")]
    Registry(#[from] std::io::Error),
    #[error("Parsing proxy address: {0}")]
    ParsingAddr(#[from] AddrParseError),
    #[error("Missing host")]
    MissingHost,
    #[error("Missing port")]
    MissingPort,
    #[error("Missing proxy config")]
    MissingProxyConfig,
    #[error("System proxy is disabled")]
    SystemProxyDisabled,
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

impl<'a> From<Proxy> for git2::ProxyOptions<'a> {
    fn from(value: Proxy) -> Self {
        let mut proxy = git2::ProxyOptions::new();

        proxy.url(&value.to_string());

        proxy
    }
}

impl FromStr for Proxy {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (auth, host) = s
            .split_once('@')
            .map_or((None, s), |(auth, host)| (Some(auth), host));

        let (username, password) = if let Some(auth) = auth {
            if let Some(auth) = auth.split_once(':') {
                (Some(auth.0.to_string()), Some(auth.1.to_string()))
            } else if auth == "currentuser" {
                panic!("sfsu does not support using the windows credentials yet");
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let host = if host == "default" {
            let (address, port) = {
                let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
                let key =
                    hklm.open_subkey("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters")?;

                if key.get_value("EnableProxy").unwrap_or(0u32) != 1 {
                    return Err(Error::SystemProxyDisabled);
                }

                let server: String = key.get_value("ProxyServer")?;

                let (host, port) = if server.is_empty() {
                    return Err(Error::MissingProxyConfig);
                } else {
                    let socket = SocketAddr::from_str(&server)?;

                    (socket.ip(), socket.port())
                };

                (host, port)
            };

            format!("{address}:{port}")
        } else {
            host.to_string()
        };

        let mut parts = host.split(':');
        let host = parts.next().ok_or(Error::MissingHost)?.to_string();
        let port = parts.next().unwrap_or("80").parse()?;

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

            if s == "default" {
                return Err(serde::de::Error::custom(
                    "default proxy is not yet supported",
                ));
            }

            Proxy::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
}
