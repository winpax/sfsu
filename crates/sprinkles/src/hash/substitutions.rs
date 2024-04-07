use std::collections::HashMap;

use derive_more::{Deref, DerefMut};
use url::Url;

use crate::hash::url::{leaf, remote_filename, strip_ext, strip_filename, strip_fragment};

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct SubstitutionMap(HashMap<String, String>);

impl SubstitutionMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn substitute(&self, builder: SubstituteBuilder, regex_escape: bool) -> String {
        builder.substitute(self, regex_escape)
    }
}

impl From<&Url> for SubstitutionMap {
    fn from(url: &Url) -> Self {
        let stripped_url = {
            let mut url = url.clone();
            strip_fragment(&mut url);
            url
        };

        let basename = remote_filename(url);

        let mut map = SubstitutionMap::new();

        map.insert("$url".into(), stripped_url.to_string());
        map.insert("$baseurl".into(), {
            let mut base_url = stripped_url.clone();
            strip_filename(&mut base_url);
            base_url.to_string()
        });
        map.insert("$basenameNoExt".into(), strip_ext(&basename).to_string());
        map.insert("$basename".into(), basename);

        if let Some(url_no_ext) = leaf(&stripped_url).as_ref().map(|fname| strip_ext(fname)) {
            map.insert("$urlNoExt".into(), url_no_ext.to_string());
        }

        todo!()
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
