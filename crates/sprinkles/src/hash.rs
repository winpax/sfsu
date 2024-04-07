use std::collections::HashMap;

use self::formats::{json::JsonError, text::TextError};

mod formats;
mod ops;

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Text error: {0}")]
    TextError(#[from] TextError),
    #[error("Json error: {0}")]
    JsonError(#[from] JsonError),
    #[error("Error parsing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Hash not found")]
    NotFound,
    #[error("Invalid hash")]
    InvalidHash,
}

pub type Result<T> = std::result::Result<T, HashError>;

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

impl TryFrom<&String> for HashType {
    type Error = HashError;

    fn try_from(value: &String) -> Result<Self> {
        match value.len() {
            64 => Ok(HashType::Sha256),
            40 => Ok(HashType::Sha1),
            32 => Ok(HashType::MD5),
            128 => Ok(HashType::Sha512),
            _ => Err(HashError::InvalidHash),
        }
    }
}

impl Hash {
    pub fn from_rdf(
        source: impl AsRef<str>,
        file_names: &[impl AsRef<str>],
    ) -> Vec<(String, Hash)> {
        formats::rdf::parse_xml(source, file_names)
            .into_iter()
            .map(|(hash_file, hash)| {
                let hash_type = HashType::try_from(&hash).unwrap_or_default();
                (hash_file, Hash { hash, hash_type })
            })
            .collect()
    }

    /// Parse a hash from a text source
    ///
    /// # Errors
    /// - If the hash is not found
    /// - If the hash is invalid
    pub fn from_text(
        source: impl AsRef<str>,
        substitutions: &HashMap<String, String>,
        regex: String,
    ) -> Result<Hash> {
        let hash =
            formats::text::parse_text(source, substitutions, regex)?.ok_or(HashError::NotFound)?;
        let hash_type = HashType::try_from(&hash)?;

        Ok(Hash { hash, hash_type })
    }

    /// Parse a hash from a json source
    ///
    /// # Errors
    /// - If the hash is not found
    /// - If the hash is invalid
    pub fn from_json(
        source: impl AsRef<[u8]>,
        substitutions: &HashMap<String, String>,
        json_path: String,
    ) -> Result<Hash> {
        let json = serde_json::from_slice(source.as_ref())?;

        let hash = formats::json::parse_json(&json, substitutions, json_path)?;
        let hash_type = HashType::try_from(&hash)?;

        Ok(Hash { hash, hash_type })
    }
}
