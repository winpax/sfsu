use std::fmt::Display;

use derive_more::{AsMut, AsRef, Deref, DerefMut};
use serde::{de::Visitor, Deserialize, Serialize};

#[macro_export]
macro_rules! wrap_bool {
    (true) => {
        NicerBool::TRUE
    };
    (false) => {
        NicerBool::FALSE
    };
    ($b:expr) => {
        NicerBool::new($b)
    };
}

pub use wrap_bool;

#[derive(Debug, Copy, Clone, AsRef, AsMut, Deref, DerefMut)]
pub struct NicerBool(bool);

impl NicerBool {
    pub const TRUE: Self = Self::new(true);
    pub const FALSE: Self = Self::new(false);

    #[must_use]
    pub const fn new(b: bool) -> Self {
        Self(b)
    }
}

impl From<bool> for NicerBool {
    fn from(b: bool) -> Self {
        Self(b)
    }
}

impl Display for NicerBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NicerBool(true) => write!(f, "Yes"),
            NicerBool(false) => write!(f, "No"),
        }
    }
}

impl Serialize for NicerBool {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NicerBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(NicerBoolVisitor)
    }
}

struct NicerBoolVisitor;
impl<'de> Visitor<'de> for NicerBoolVisitor {
    type Value = NicerBool;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string equal to either 'Yes' or 'No'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "Yes" => Ok(NicerBool::TRUE),
            "No" => Ok(NicerBool::FALSE),
            _ => Err(E::custom(format!("expected 'Yes' or 'No', found '{v}'"))),
        }
    }
}
