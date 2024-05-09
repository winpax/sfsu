use serde::{de::Visitor, Deserialize, Serialize};

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
/// The path to use for Scoop instead of the system path
pub struct IsolatedPath(PathBuf);

impl From<PathBuf> for IsolatedPath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&PathBuf> for IsolatedPath {
    fn from(path: &PathBuf) -> Self {
        Self(path.to_path_buf())
    }
}

impl From<&Path> for IsolatedPath {
    fn from(path: &Path) -> Self {
        Self(path.to_path_buf())
    }
}

impl AsRef<Path> for IsolatedPath {
    fn as_ref(&self) -> &Path {
        self.0.as_path()
    }
}

impl Serialize for IsolatedPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let path = self.0.as_path();
        if let Some(scoop_path_env) = crate::env::paths::scoop_path() {
            if path == scoop_path_env {
                return serializer.serialize_bool(true);
            }
        }

        serializer.serialize_str(&path.display().to_string())
    }
}

struct IsolatedPathVisitor;
impl<'de> Visitor<'de> for IsolatedPathVisitor {
    type Value = IsolatedPath;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("true or a custom variable name")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v {
            Ok(IsolatedPath(crate::env::paths::scoop_path().unwrap()))
        } else {
            Err(serde::de::Error::custom("expected true"))
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "true" => self.visit_bool(true),
            "false" => self.visit_bool(false),
            _ => Ok(IsolatedPath(PathBuf::from(v))),
        }
    }
}

impl<'de> Deserialize<'de> for IsolatedPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(IsolatedPathVisitor)
    }
}
