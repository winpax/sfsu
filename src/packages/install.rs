use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Manifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The bucket the package was installed from
    pub bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<Architecture>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Architecture {
    #[default]
    Unknown,
    X86,
    X64,
}

impl Serialize for Architecture {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Architecture::Unknown => serializer.serialize_none(),
            Architecture::X86 => serializer.serialize_str("32bit"),
            Architecture::X64 => serializer.serialize_str("64bit"),
        }
    }
}

impl<'de> Deserialize<'de> for Architecture {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Value = Deserialize::deserialize(deserializer)?;

        match v {
            Value::String(s) => match s.as_str() {
                "32bit" => Ok(Architecture::X86),
                "64bit" => Ok(Architecture::X64),
                _ => Ok(Architecture::Unknown),
            },
            _ => Ok(Architecture::Unknown),
        }
    }
}

impl Manifest {
    pub fn get_source(&self) -> String {
        match (&self.bucket, &self.url) {
            (Some(bucket), None) => bucket.to_string(),
            (None, Some(url)) => url.to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Architecture, Manifest};

    #[test]
    fn test_install_manifest_serde() {
        // Formatted the same as serde_json will output
        const MANIFEST: &str = r#"{"bucket":"main","architecture":"64bit"}"#;

        let zig_manifest = Manifest {
            bucket: Some("main".to_string()),
            hold: None,
            url: None,
            architecture: Some(Architecture::X64),
        };

        let deserialized_manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        assert_eq!(deserialized_manifest, zig_manifest);

        let serialized_manifest = serde_json::to_string(&zig_manifest).unwrap();

        assert_eq!(serialized_manifest, MANIFEST);
    }

    #[test]
    fn test_held_install_manifest_serde() {
        // Formatted the same as serde_json will output
        const MANIFEST: &str = r#"{"bucket":"main","hold":true,"architecture":"64bit"}"#;

        let zig_manifest = Manifest {
            bucket: Some("main".to_string()),
            hold: Some(true),
            url: None,
            architecture: Some(Architecture::X64),
        };

        let deserialized_manifest: Manifest = serde_json::from_str(MANIFEST).unwrap();

        assert_eq!(deserialized_manifest, zig_manifest);

        let serialized_manifest = serde_json::to_string(&zig_manifest).unwrap();

        assert_eq!(serialized_manifest, MANIFEST);
    }
}
