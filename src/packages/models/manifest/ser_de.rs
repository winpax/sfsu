use std::str::FromStr;

use serde::{
    Deserialize, Deserializer,
    de::{Error, Unexpected},
};

use crate::hash::Hash;

use super::SingleOrArray;

fn parse_hash<'de, D: Deserializer<'de>>(value: &str) -> Result<Hash, D::Error> {
    Hash::from_str(value).map_err(|_| Error::invalid_value(
                Unexpected::Str(value),
                &"a valid sha512, sha256, sha1 or md5 hash, with a prefix for any type other than sha256",
            )
)
}

pub(super) fn deserialize_hash<'de, D: Deserializer<'de>>(
    data: D,
) -> Result<Option<SingleOrArray<Hash>>, D::Error> {
    // This is an insanely inelegant workaround
    // blame serde its not my fault

    let value = serde_json::Value::deserialize(data)?;

    #[allow(if_let_rescope)]
    if let Some(real_value) = value.as_str() {
        if real_value.is_empty() {
            return Ok(None);
        }

        let hash = parse_hash::<D>(real_value)?;

        Ok(Some(SingleOrArray::Single(hash)))
    } else if let Some(array) = value.as_array() {
        let mut hashes = Vec::new();

        for value in array {
            if let Some(real_value) = value.as_str() {
                let hash = parse_hash::<D>(real_value)?;

                hashes.push(hash);
            } else {
                return Err(Error::invalid_value(
                    Unexpected::Other("array contained a non-string value"),
                    &"a valid sha512, sha256, sha1 or md5 hash, with a prefix for any type other than sha256",
                ));
            }
        }

        Ok(Some(SingleOrArray::Array(hashes)))
    } else {
        Err(Error::custom(
            "data did not match any variant of untagged enum SingleOrArray",
        ))
    }
}
