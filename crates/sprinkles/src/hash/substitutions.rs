use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use url::Url;

use crate::{
    hash::url_ext::UrlExt,
    packages::models::manifest::{AliasArray, Installer, StringArray, TOrArrayOfTs},
    version::Version,
};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct SubstitutionMap(HashMap<String, String>);

impl SubstitutionMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn from_all(version: &Version, url: &Url) -> Self {
        let mut map = Self::new();

        map.append_version(version);
        map.append_url(url);

        map
    }

    pub fn substitute(&self, builder: SubstituteBuilder, regex_escape: bool) -> String {
        builder.substitute(self, regex_escape)
    }

    /// Append version information to the map
    pub fn append_version(&mut self, version: &Version) {
        self.extend(version.submap().0);
    }

    pub fn append_url(&mut self, url: &Url) {
        self.extend(url.submap().0);
    }
}

impl Default for SubstitutionMap {
    fn default() -> Self {
        Self::new()
    }
}

pub enum SubstituteBuilder {
    String(String),
}

impl SubstituteBuilder {
    pub fn substitute(self, params: &SubstitutionMap, regex_escape: bool) -> String {
        match self {
            SubstituteBuilder::String(entity) => {
                let mut new_entity = entity;

                for key in params.keys() {
                    let value = params.get(key).unwrap();

                    if regex_escape {
                        new_entity = new_entity.replace(key, &regex::escape(value));
                    } else {
                        new_entity = new_entity.replace(key, value);
                    }
                }

                new_entity
            }
        }
    }
}

pub trait Substitute {
    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool)
    where
        Self: Clone,
    {
        let substituted = self.clone().into_substituted(params, regex_escape);

        *self = substituted;
    }

    #[must_use]
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self;
}

impl Substitute for String {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        SubstituteBuilder::String(self).substitute(params, regex_escape)
    }
}

impl Substitute for TOrArrayOfTs<String> {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        self.map(|s| s.into_substituted(params, regex_escape))
    }
}

impl Substitute for Vec<String> {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        self.into_iter()
            .map(|s| s.into_substituted(params, regex_escape))
            .collect()
    }
}

impl Substitute for Vec<Vec<String>> {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        self.into_iter()
            .map(|s| s.into_substituted(params, regex_escape))
            .collect()
    }
}

impl Substitute for AliasArray<String> {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        match self {
            AliasArray::NestedArray(StringArray::Single(s)) => AliasArray::NestedArray(
                StringArray::Single(s.into_substituted(params, regex_escape)),
            ),
            AliasArray::NestedArray(StringArray::Array(s)) => {
                AliasArray::NestedArray(StringArray::Array(
                    s.into_iter()
                        .map(|s| s.into_substituted(params, regex_escape))
                        .collect(),
                ))
            }
            AliasArray::AliasArray(s) => AliasArray::AliasArray(
                s.into_iter()
                    .map(|s| s.into_substituted(params, regex_escape))
                    .collect(),
            ),
        }
    }
}

impl Substitute for Installer {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        Installer {
            file: self.file.map(|s| s.into_substituted(params, regex_escape)),
            comment: self
                .comment
                .map(|s| s.into_substituted(params, regex_escape)),
            args: self.args.map(|s| s.into_substituted(params, regex_escape)),
            keep: self.keep,
            script: self
                .script
                .map(|s| s.into_substituted(params, regex_escape)),
        }
    }
}
