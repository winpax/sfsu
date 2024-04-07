use std::collections::HashMap;

use serde_json::Value;
use serde_json_path::{JsonPath, NodeList};

use crate::ops::Substitute;

#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid JSON path: {0}")]
    JsonPath(#[from] serde_json_path::ParseError),

    #[error("Matching path not found")]
    NotFound,
}

pub fn parse_json(
    json: &Value,
    substitutions: HashMap<String, String>,
    jp: String,
) -> Result<String, JsonError> {
    // let json: Value = serde_json::from_slice(source)?;

    let hashes = query_jp(json, jp, substitutions)?;

    hashes
        .first()
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or(JsonError::NotFound)
}

fn query_jp(
    json: &Value,
    jp: String,
    substitutions: HashMap<String, String>,
) -> Result<NodeList<'_>, JsonError> {
    let jp = {
        let regex_escape = jp.contains("=~");
        jp.into_substituted(&substitutions, regex_escape)
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

    use crate::requests::BlockingClient;

    #[test]
    fn test_finding_json_hashes() -> anyhow::Result<()> {
        const URL: &str = "https://api.azul.com/zulu/download/community/v1.0/bundles/latest/?jdk_version=&bundle_type=jdk&features=&javafx=false&ext=zip&os=windows&arch=x86&hw_bitness=64";

        let substitutions = HashMap::new();
        let jp = "$.sha256_hash".to_string();

        let source = BlockingClient::new().get(URL).send()?.bytes()?;
        let json: Value = serde_json::from_slice(&source)?;

        let actual_hash = json
            .get("sha256_hash")
            .and_then(|v| v.as_str())
            .expect("sha256 hash in json download");

        let hashes = parse_json(&json, substitutions, jp).unwrap();

        assert_eq!(actual_hash, hashes);

        Ok(())
    }
}
