use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum JsonError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn parse_json(
    source: &[u8],
    substitutions: HashMap<String, String>,
    json_path: String,
) -> Result<String, JsonError> {
    let json: Value = serde_json::from_slice(source)?;

    unimplemented!()
}

fn json_path(json: String, json_path: String, substitutions: HashMap<String, String>) -> String {
    unimplemented!()
}
