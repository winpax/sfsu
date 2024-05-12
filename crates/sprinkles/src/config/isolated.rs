//! The path to use for Scoop instead of the system path

use std::path::PathBuf;

use serde::de::Visitor;

pub fn serialize<S>(path: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(path) = path {
        if let Some(scoop_path_env) = crate::env::paths::scoop_path() {
            if path == &scoop_path_env {
                return serializer.serialize_bool(true);
            }
        }

        serializer.serialize_str(&path.display().to_string())
    } else {
        serializer.serialize_none()
    }
}

struct IsolatedPathVisitor;
impl<'de> Visitor<'de> for IsolatedPathVisitor {
    type Value = Option<PathBuf>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("true or a custom variable name")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v {
            Ok(Some(crate::env::paths::scoop_path().unwrap()))
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
            _ => Ok(Some(PathBuf::from(v))),
        }
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_option(IsolatedPathVisitor)
}
