use std::collections::HashMap;

pub enum SubstituteBuilder {
    String(String),
}

impl SubstituteBuilder {
    pub fn substitute(self, params: HashMap<String, String>, regex_escape: bool) -> String {
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
    fn substitute(&mut self, params: HashMap<String, String>, regex_escape: bool)
    where
        Self: Clone;

    fn into_substituted(self, params: HashMap<String, String>, regex_escape: bool) -> Self;
}

impl Substitute for String {
    fn into_substituted(self, params: HashMap<String, String>, regex_escape: bool) -> Self {
        SubstituteBuilder::String(self).substitute(params, regex_escape)
    }

    fn substitute(&mut self, params: HashMap<String, String>, regex_escape: bool) {
        let substituted = self.clone().into_substituted(params, regex_escape);

        *self = substituted;
    }
}
