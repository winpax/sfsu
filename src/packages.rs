use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::get_scoop_path;

pub trait FromPath
where
    Self: Default,
{
    /// Convert a path into a manifest
    ///
    /// # Errors
    /// - The file does not exist
    /// - The file was not valid UTF-8
    fn from_path(path: impl AsRef<Path>) -> std::io::Result<Self>
    where
        Self: for<'a> Deserialize<'a>,
    {
        let path = path.as_ref();
        let mut file = File::open(path)?;
        let mut contents = String::new();

        file.read_to_string(&mut contents)?;

        Ok(
            serde_json::from_str(contents.trim_start_matches('\u{feff}')).unwrap_or_else(|err| {
                println!("Error parsing manifest: {}", path.display());
                println!("{err}");

                Default::default()
            }),
        )
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct License {
    identifier: String,
    url: Option<String>,
}

impl<'de> Deserialize<'de> for License {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v: Value = Deserialize::deserialize(deserializer)?;

        match v {
            Value::String(identifier) => Ok(License {
                identifier,
                url: None,
            }),
            Value::Object(license) => {
                let id = license
                    .get("identifier")
                    .and_then(serde_json::Value::as_str)
                    .expect("string identifier");

                let url = license
                    .get("url")
                    .and_then(serde_json::Value::as_str)
                    .map(std::string::ToString::to_string);

                Ok(License {
                    identifier: id.to_owned(),
                    url,
                })
            }
            _ => panic!("Invalid license in manifest"),
        }
    }
}

impl std::fmt::Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier)?;

        if let Some(url) = &self.url {
            write!(f, " | {url}")?;
        }

        writeln!(f)
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct Manifest {
    #[serde(default = "default_version")]
    /// The version of the package
    pub version: String,
    #[serde(
        default,
        deserialize_with = "ok_or_default",
        skip_serializing_if = "Option::is_none"
    )]
    /// The description of the package
    pub description: Option<String>,
    #[serde(
        default,
        deserialize_with = "ok_or_default",
        skip_serializing_if = "Option::is_none"
    )]
    /// The homepage of the package
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// The license of the package,
    pub license: Option<License>,
}

fn default_version() -> String {
    "Invalid Manifest".to_string()
}

fn ok_or_default<'a, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'a> + Default,
    D: serde::Deserializer<'a>,
{
    let v: Value = Deserialize::deserialize(deserializer)?;
    Ok(T::deserialize(v).unwrap_or_default())
}

impl FromPath for Manifest {}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct InstallManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The bucket the package was installed from
    pub bucket: Option<String>,
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

impl InstallManifest {
    pub fn get_source(&self) -> String {
        match (&self.bucket, &self.url) {
            (Some(bucket), None) => bucket.to_string(),
            (None, Some(url)) => url.to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

impl FromPath for InstallManifest {}

/// Check if the manifest path is installed, and optionally confirm the bucket
///
/// # Panics
/// - The file was not valid UTF-8
pub fn is_installed(manifest_name: impl AsRef<Path>, bucket: Option<impl AsRef<str>>) -> bool {
    let scoop_path = get_scoop_path();
    let installed_path = scoop_path
        .join("apps")
        .join(manifest_name)
        .join("current/install.json");

    match InstallManifest::from_path(installed_path) {
        Ok(manifest) => {
            if let Some(bucket) = bucket {
                manifest.get_source() == bucket.as_ref()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{Architecture, InstallManifest, Manifest};

    #[test]
    fn test_install_manifest_serde() {
        // Formatted the same as serde_json will output
        const MANIFEST: &str = r#"{"bucket":"main","architecture":"64bit"}"#;

        let zig_manifest = InstallManifest {
            bucket: Some("main".to_string()),
            url: None,
            architecture: Some(Architecture::X64),
        };

        let deserialized_manifest: InstallManifest = serde_json::from_str(MANIFEST).unwrap();

        assert_eq!(deserialized_manifest, zig_manifest);

        let serialized_manifest = serde_json::to_string(&zig_manifest).unwrap();

        assert_eq!(serialized_manifest, MANIFEST);
    }

    #[test]
    fn test_manifest_serde() {
        const MANIFEST: &str = r#"{"version":"0.10.1","description":"General-purpose programming language designed for robustness, optimality, and maintainability.","homepage":"https://ziglang.org/","license":"MIT","suggest":{"vcredist":"extras/vcredist2022"},"architecture":{"64bit":{"url":"https://ziglang.org/download/0.10.1/zig-windows-x86_64-0.10.1.zip","hash":"5768004e5e274c7969c3892e891596e51c5df2b422d798865471e05049988125","extract_dir":"zig-windows-x86_64-0.10.1"},"arm64":{"url":"https://ziglang.org/download/0.10.1/zig-windows-aarch64-0.10.1.zip","hash":"ece93b0d77b2ab03c40db99ef7ccbc63e0b6bd658af12b97898960f621305428","extract_dir":"zig-windows-aarch64-0.10.1"}},"bin":"zig.exe","checkver":{"url":"https://ziglang.org/download/","regex":">([\\d.]+)</h"},"autoupdate":{"architecture":{"64bit":{"url":"https://ziglang.org/download/$version/zig-windows-x86_64-$version.zip","extract_dir":"zig-windows-x86_64-$version"},"arm64":{"url":"https://ziglang.org/download/$version/zig-windows-aarch64-$version.zip","extract_dir":"zig-windows-aarch64-$version"}},"hash":{"url":"https://ziglang.org/download/","regex":"(?sm)$basename.*?$sha256"}}}"#;

        let deserialized: Manifest = serde_json::from_str(MANIFEST).unwrap();

        dbg!(deserialized);
    }
}
