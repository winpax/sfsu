use serde_json::Value;
use serde_json_path::{JsonPath, NodeList};

use crate::hash::substitutions::{Substitute, SubstitutionMap};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid JSON path: {0}")]
    JsonPath(#[from] serde_json_path::ParseError),

    #[error("Matching path not found")]
    NotFound,
}

pub fn parse_json(
    json: &Value,
    substitutions: &SubstitutionMap,
    jp: impl AsRef<str>,
) -> Result<String, Error> {
    // let json: Value = serde_json::from_slice(source)?;

    let hashes = query_jp(json, jp.as_ref(), substitutions)?;

    hashes
        .first()
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string)
        .ok_or(Error::NotFound)
}

fn query_jp<'a>(
    json: &'a Value,
    jp: &str,
    substitutions: &SubstitutionMap,
) -> Result<NodeList<'a>, Error> {
    let jp = {
        let regex_escape = jp.contains("=~");
        jp.to_string().into_substituted(substitutions, regex_escape)
    };

    let path = JsonPath::parse(&jp)?;

    Ok(path.query(json))

    // .and_then(|v| v.as_str())
    //     .map(|s| s.to_string())
    //     .ok_or(JsonError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::requests::Client;

    #[test]
    fn test_finding_json_hashes() -> anyhow::Result<()> {
        const URL: &str = "https://api.azul.com/zulu/download/community/v1.0/bundles/latest/?jdk_version=&bundle_type=jdk&features=&javafx=false&ext=zip&os=windows&arch=x86&hw_bitness=64";

        let substitutions = SubstitutionMap::new();
        let jp = "$.sha256_hash".to_string();

        let source = Client::blocking().get(URL).send()?.bytes()?;
        let json: Value = serde_json::from_slice(&source)?;

        let actual_hash = json
            .get("sha256_hash")
            .and_then(|v| v.as_str())
            .expect("sha256 hash in json download");

        let hashes = parse_json(&json, &substitutions, jp).unwrap();

        assert_eq!(actual_hash, hashes);

        Ok(())
    }
}
