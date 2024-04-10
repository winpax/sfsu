use itertools::Itertools;
use serde_json::Value;
use sxd_document::parser;
use sxd_xpath::{evaluate_xpath, Value};

use crate::hash::{substitutions::SubstitutionMap, Hash};

#[derive(Debug, thiserror::Error)]
pub enum XMLError {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::DeError),
}

pub fn parse_xml(
    source: impl AsRef<[u8]>,
    substitutions: &SubstitutionMap,
    xpath: impl AsRef<str>,
) -> Result<Hash, XMLError> {
    let path_segments = {
        let mut segments = xpath.as_ref().split('/').collect_vec();

        if let Some(first) = segments.first() {
            if first.is_empty() {
                segments.remove(0);
            }
        }

        segments
    };

    let data: Value = quick_xml::de::from_reader(source.as_ref())?;

    todo!()
}
