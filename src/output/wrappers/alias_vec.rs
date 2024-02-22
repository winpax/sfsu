use std::fmt::Display;

use derive_more::{Deref, DerefMut};
use serde::Serialize;

#[derive(Debug, Default, Clone, Deref, DerefMut)]
#[must_use]
pub struct AliasVec<T>(Vec<Vec<T>>);

impl<T: Display> AliasVec<T> {
    pub fn from_vec(vec: Vec<Vec<T>>) -> Self {
        Self(vec)
    }

    pub fn from_shortcuts(vec: Option<Vec<Vec<T>>>) -> Self {
        Self(vec.unwrap_or_default())
    }
}

impl<T: Display> Display for AliasVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;

        for alias_cfg in &self.0 {
            if !first {
                write!(f, ", ")?;
            }

            // let value = alias[0];
            let alias = &alias_cfg[1];

            alias.fmt(f)?;
            first = false;
        }

        Ok(())
    }
}

impl<T: Serialize> Serialize for AliasVec<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}
