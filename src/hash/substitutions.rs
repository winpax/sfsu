//! Substitution helpers

use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use url::Url;

use crate::{
    hash::url_ext::UrlExt,
    packages::models::manifest::{Installer, NestedArray, SingleOrArray},
    version::Version,
};

fn replace_in_place(string: &mut String, from: &str, to: &str) {
    for (start, part) in string.clone().match_indices(from) {
        string.replace_range(start..start + part.len(), to);
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
/// Substitution map
pub struct SubstitutionMap(HashMap<String, String>);

impl SubstitutionMap {
    #[must_use]
    /// Create a new substitution map
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[must_use]
    /// Create a new substitution map from the version and url
    pub fn from_all(version: &Version, url: &Url) -> Self {
        let mut map = Self::new();

        map.append_version(version);
        map.append_url(url);

        map
    }

    /// Substitute the string with the substitution map
    pub fn substitute(&self, string: &mut String, regex_escape: bool) {
        SubstituteBuilder::String(string).substitute(self, regex_escape);
    }

    /// Append version information to the map
    pub fn append_version(&mut self, version: &Version) {
        self.extend(version.submap().0);
    }

    /// Append the url to the substitution map
    pub fn append_url(&mut self, url: &Url) {
        self.extend(url.submap().0);
    }
}

impl Default for SubstitutionMap {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for SubstitutionMap {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

impl<'a> From<HashMap<&'a str, String>> for SubstitutionMap {
    fn from(map: HashMap<&'a str, String>) -> Self {
        Self(map.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
}

/// Substitute builder
pub enum SubstituteBuilder<'a> {
    /// Substitute a string
    String(&'a mut String),
}

impl SubstituteBuilder<'_> {
    /// Substitute the builder with the substitution map
    pub fn substitute(self, params: &SubstitutionMap, regex_escape: bool) {
        match self {
            SubstituteBuilder::String(new_entity) => {
                for (key, value) in params.iter() {
                    if regex_escape {
                        replace_in_place(new_entity, key, &regex::escape(value));
                    } else {
                        replace_in_place(new_entity, key, value);
                    }
                }
            }
        }
    }
}

/// Substitute trait
///
/// This trait is used to substitute strings with a [`SubstitutionMap`]
/// It is implemented for [`String`], [`SingleOrArray<String>`], and [`Installer`]
pub trait Substitute {
    /// Substitute the entity with the substitution map
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool);

    #[must_use]
    /// Substitute the entity with the substitution map
    fn into_substituted(mut self, params: &SubstitutionMap, regex_escape: bool) -> Self
    where
        Self: Clone,
    {
        self.substitute(params, regex_escape);
        self
    }
}

impl Substitute for String {
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool) {
        SubstituteBuilder::String(self).substitute(params, regex_escape);
    }
}

impl<T: Substitute> Substitute for SingleOrArray<T> {
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool) {
        match self {
            SingleOrArray::Single(s) => s.substitute(params, regex_escape),
            SingleOrArray::Array(a) => {
                for s in a.iter_mut() {
                    s.substitute(params, regex_escape);
                }
            }
        }
    }
}

impl<T: Substitute> Substitute for Vec<T> {
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool) {
        for s in self.iter_mut() {
            s.substitute(params, regex_escape);
        }
    }
}

impl<T: Substitute> Substitute for NestedArray<T> {
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool) {
        match self {
            NestedArray::NestedArray(SingleOrArray::Single(s)) => {
                s.substitute(params, regex_escape);
            }
            NestedArray::NestedArray(SingleOrArray::Array(s)) => s
                .iter_mut()
                .for_each(|s| s.substitute(params, regex_escape)),
            NestedArray::AliasArray(s) => s
                .iter_mut()
                .for_each(|s| s.substitute(params, regex_escape)),
        }
    }
}

impl Substitute for Installer {
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool) {
        if let Some(s) = self.file.as_mut() {
            s.substitute(params, regex_escape);
        }

        if let Some(s) = self.comment.as_mut() {
            s.substitute(params, regex_escape);
        }

        if let Some(s) = self.args.as_mut() {
            s.substitute(params, regex_escape);
        }

        if let Some(s) = self.script.as_mut() {
            s.substitute(params, regex_escape);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::replace_in_place;

    #[test]
    fn test_replace_in_place() {
        let mut string = String::from("Hello, world!");
        let should_be = string.replace("world", "rust");

        replace_in_place(&mut string, "world", "rust");

        assert_eq!(string, should_be);
    }
}
