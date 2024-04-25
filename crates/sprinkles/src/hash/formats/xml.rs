use sxd_document::parser;
use sxd_xpath::{evaluate_xpath, Value};

use crate::hash::substitutions::{Substitute, SubstitutionMap};

#[derive(Debug, thiserror::Error)]
pub enum XMLError {
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("XML Parsing: {0}")]
    SXDXml(#[from] sxd_document::parser::Error),
    #[error("XML XPath: {0}")]
    XPath(#[from] sxd_xpath::Error),
    #[error("Hash not found")]
    NotFound,
    #[error("Invalid value")]
    InvalidValue,
}

pub fn parse_xml(
    source: impl AsRef<str>,
    substitutions: &SubstitutionMap,
    xpath: impl AsRef<str>,
) -> Result<String, XMLError> {
    let mut xpath = xpath.as_ref().to_string();
    xpath.substitute(substitutions, false);

    let pkg = parser::parse(source.as_ref())?;
    let doc = pkg.as_document();

    let value = evaluate_xpath(&doc, xpath.as_ref())?;

    let hash = match value {
        Value::Nodeset(nodes) => {
            let node = nodes.iter().last().ok_or(XMLError::NotFound)?;

            node.string_value()
        }
        Value::String(text) => text,
        _ => return Err(XMLError::InvalidValue),
    };

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_in_xml() -> anyhow::Result<()> {
        const EXAMPLE_XML: &str = r#"
            <assembly>
                <description>sfsu</description>
                <compatibility>
                    <application>
                        <!-- Windows 10 and Windows 11 -->
                        <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" />
                        <!-- Windows 8.1 -->
                        <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}" />
                        <!-- Windows 8 -->
                        <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}" />
                        <!-- Windows 7 -->
                        <supportedOS Id="{35138b9a-5d96-4fbd-8e2d-a2440225f93a}" />
                        <!-- Windows Vista -->
                        <supportedOS Id="{e2011457-1546-43c5-a5fe-008deee3d3f0}" />
                    </application>
                </compatibility>
            </assembly>
        "#;

        let mut submap = SubstitutionMap::new();

        submap.insert("$finalKey".to_string(), "supportedOS".to_string());

        let hash = parse_xml(
            EXAMPLE_XML,
            &submap,
            "/assembly/compatibility/application/$finalKey[last() - 1]/@Id",
        )?;

        assert_eq!(hash, "{35138b9a-5d96-4fbd-8e2d-a2440225f93a}");

        Ok(())
    }
}
