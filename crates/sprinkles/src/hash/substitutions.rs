use std::collections::HashMap;

use derive_more::{Deref, DerefMut};

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
