use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use regex::Regex;
use url::Url;

use crate::{
    hash::url_ext::{strip_ext, UrlExt},
    version::Version,
};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct SubstitutionMap(HashMap<String, String>);

impl SubstitutionMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn substitute(&self, builder: SubstituteBuilder, regex_escape: bool) -> String {
        builder.substitute(self, regex_escape)
    }

    /// Append version information to the map
    pub fn append_version(&mut self, version: &Version) {
        self.insert("$version".into(), version.as_str().to_string());
        self.insert("$dotVersion".into(), version.dot_version().to_string());
        self.insert(
            "$underscoreVersion".into(),
            version.underscore_version().to_string(),
        );
        self.insert("$dashVersion".into(), version.dash_version().to_string());
        self.insert("$cleanVersion".into(), version.clean_version().to_string());

        if let Ok(parsed) = version.parse() {
            self.insert("$majorVersion".into(), parsed.major().to_string());

            if let Some(minor) = parsed.minor() {
                self.insert("$minorVersion".into(), minor.to_string());
            }
            if let Some(patch) = parsed.patch() {
                self.insert("$patchVersion".into(), patch.to_string());
            }
            if let Some(build) = parsed.build() {
                self.insert("$buildVersion".into(), build.clone());
            }
            if let Some(pre_release) = parsed.pre_release() {
                self.insert("$preReleaseVersion".into(), pre_release.clone());
            }
        }

        let matches_regex = Regex::new(r"(?<head>\d+\.\d+(?:\.\d+)?)(?<tail>.*)").unwrap();
        if let Some(captures) = matches_regex.captures(version.as_str()) {
            // The following two `if let` statements in theory should always be true
            // But to avoid a panic in case of a bug, we are using `if let` instead of `unwrap`

            if let Some(head) = captures.name("head") {
                self.insert("$matchHead".into(), head.as_str().to_string());
            }

            if let Some(tail) = captures.name("tail") {
                self.insert("$matchTail".into(), tail.as_str().to_string());
            }
        }
    }

    pub fn append_url(&mut self, url: &Url) {
        let map = SubstitutionMap::from(url);
        self.extend(map.0);
    }
}

impl From<&Url> for SubstitutionMap {
    fn from(url: &Url) -> Self {
        let stripped_url = {
            let mut url = url.clone();
            url.strip_fragment();
            url
        };

        let basename = url.remote_filename();

        let mut map = SubstitutionMap::new();

        map.insert("$url".into(), stripped_url.to_string());
        map.insert("$baseurl".into(), {
            let mut base_url = stripped_url.clone();
            base_url.strip_filename();
            base_url.to_string()
        });
        map.insert("$basenameNoExt".into(), strip_ext(&basename).to_string());
        map.insert("$basename".into(), basename);

        if let Some(url_no_ext) = stripped_url.leaf().as_ref().map(|fname| strip_ext(fname)) {
            map.insert("$urlNoExt".into(), url_no_ext.to_string());
        }

        map
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
        Self: Clone;

    #[must_use]
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self;
}

impl Substitute for String {
    fn into_substituted(self, params: &SubstitutionMap, regex_escape: bool) -> Self {
        SubstituteBuilder::String(self).substitute(params, regex_escape)
    }

    fn substitute(&mut self, params: &SubstitutionMap, regex_escape: bool) {
        let substituted = self.clone().into_substituted(params, regex_escape);

        *self = substituted;
    }
}
