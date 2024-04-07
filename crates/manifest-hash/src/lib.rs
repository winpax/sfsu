use std::collections::HashMap;

mod formats;
mod ops;

#[macro_use]
extern crate log;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    hash: String,
    hash_type: HashType,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum HashType {
    Sha512,
    #[default]
    Sha256,
    Sha1,
    MD5,
}

impl Hash {
    pub fn from_rdf(
        source: impl AsRef<str>,
        file_names: &[impl AsRef<str>],
    ) -> Vec<(String, Self)> {
        formats::rdf::parse_xml(source, file_names)
            .into_iter()
            .map(|(hash_file, hash)| {
                (
                    hash_file,
                    Self {
                        hash,
                        hash_type: HashType::Sha256,
                    },
                )
            })
            .collect()
    }

    pub fn from_text(
        source: impl AsRef<str>,
        substitutions: HashMap<String, String>,
        regex: String,
    ) -> Vec<(String, Self)> {
        unimplemented!(
            "fuck this its so hard and for what like just provide your hashes its so fucking easy LOOKING AT YOU MYSQL FUCK YOU"
        );
        formats::text::parse_text(source, substitutions, regex);
    }

    pub fn from_json(
        source: impl AsRef<[u8]>,
        substitutions: HashMap<String, String>,
        json_path: String,
    ) -> Result<(), formats::json::JsonError> {
        let json = serde_json::from_slice(source.as_ref())?;

        formats::json::parse_json(json, substitutions, json_path)?;

        todo!()
    }
}
