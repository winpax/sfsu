use std::io::{BufRead, Read};

use formats::{json::JsonError, text::TextError};
use itertools::Itertools;
use reqwest::header::{HeaderMap, HeaderValue};
use substitutions::SubstitutionMap;
use url::Url;

use crate::{
    hash::url_ext::UrlExt,
    packages::{
        arch_field,
        manifest::{
            AutoupdateConfig, HashExtractionOrArrayOfHashExtractions, HashMode as ManifestHashMode,
        },
        Manifest, MergeDefaults,
    },
};

use self::substitutions::Substitute;

mod formats;
mod substitutions;
mod url_ext;

#[derive(Debug, thiserror::Error)]
pub enum HashError {
    #[error("Text error: {0}")]
    TextError(#[from] TextError),
    #[error("Json error: {0}")]
    JsonError(#[from] JsonError),
    #[error("RDF error: {0}")]
    RDFError(#[from] formats::rdf::RDFError),
    #[error("XML error: {0}")]
    XMLError(#[from] formats::xml::XMLError),
    #[error("Error parsing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Failed to parse url: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Error downloading hash")]
    HashDownloading(#[from] reqwest::Error),
    #[error("Hash not found")]
    NotFound,
    #[error("Missing download url(s) in manifest")]
    UrlNotFound,
    #[error("Invalid hash")]
    InvalidHash,
    #[error("Missing autoupdate filter")]
    MissingAutoupdate,
    #[error("Cannot determine hash mode")]
    HashMode,
    #[error("Missing hash extraction object")]
    MissingHashExtraction,
    #[error("Hash extraction url where there should be a hash extraction object. This is a bug, please report it.")]
    HashExtractionUrl,
    #[error("Missing part of hash extraction object, where it should exist. This is a bug, please report it.")]
    MissingExtraction,
}

pub type Result<T> = std::result::Result<T, HashError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    hash: String,
    hash_type: HashType,
}

impl Hash {
    #[must_use]
    pub fn hash(&self) -> &str {
        &self.hash
    }

    #[must_use]
    pub fn hash_type(&self) -> HashType {
        self.hash_type
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.hash_type {
            HashType::Sha512 => "sha512:",
            HashType::Sha256 => "",
            HashType::Sha1 => "sha1:",
            HashType::MD5 => "md5:",
        };

        write!(f, "{prefix}")?;
        write!(f, "{}", self.hash)
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashMode {
    HashUrl,
    Download,
    Extract(String),
    Json(String),
    Xpath(String),
    Fosshub,
    Metalink,
    Rdf,
    Sourceforge,
}

impl HashMode {
    #[must_use]
    #[deprecated(note = "Does not handle Sourceforge or Fosshub")]
    /// Get a [`HashMode`] from an [`AutoupdateConfig`]
    pub fn from_autoupdate_config(config: &AutoupdateConfig) -> Option<Self> {
        let hash = config.hash.as_ref()?;

        if let HashExtractionOrArrayOfHashExtractions::Url(_) = hash {
            return Some(HashMode::Download);
        }

        if let HashExtractionOrArrayOfHashExtractions::HashExtraction(hash_cfg) = hash {
            let mode = hash_cfg
                .mode
                .and_then(|mode| match mode {
                    ManifestHashMode::Download => Some(HashMode::Download),
                    ManifestHashMode::Fosshub => Some(HashMode::Fosshub),
                    ManifestHashMode::Sourceforge => Some(HashMode::Sourceforge),
                    ManifestHashMode::Metalink => Some(HashMode::Metalink),
                    ManifestHashMode::Rdf => Some(HashMode::Rdf),
                    _ => None,
                })
                .or_else(|| {
                    if let Some(regex) = &hash_cfg.regex {
                        return Some(HashMode::Extract(regex.clone()));
                    }
                    if let Some(regex) = &hash_cfg.find {
                        return Some(HashMode::Extract(regex.clone()));
                    }

                    if let Some(jsonpath) = &hash_cfg.jsonpath {
                        return Some(HashMode::Json(jsonpath.clone()));
                    }
                    if let Some(jsonpath) = &hash_cfg.jp {
                        return Some(HashMode::Json(jsonpath.clone()));
                    }

                    if let Some(xpath) = &hash_cfg.xpath {
                        return Some(HashMode::Xpath(xpath.clone()));
                    }

                    None
                });

            return if let Some(mode) = mode {
                Some(mode)
            } else {
                dbg!(&hash_cfg.url);
                Some(HashMode::HashUrl)
            };
        }

        todo!("Handle array of hash extractions")
    }
}

impl Hash {
    /// Get a hash for an app
    ///
    /// # Errors
    /// - If the hash is not found
    /// - If the hash is invalid
    /// - If the hash mode is invalid
    /// - If the URL is invalid
    /// - If the source is invalid
    /// - If the JSON is invalid
    /// - If the hash is not found in the headers
    /// - If the hash is not found in the RDF
    /// - If the hash is not found in the XML
    /// - If the hash is not found in the text
    /// - If the hash is not found in the JSON
    pub fn get_for_app(manifest: &Manifest) -> Result<Hash> {
        let autoupdate_config = manifest
            .autoupdate
            .as_ref()
            .and_then(|autoupdate| autoupdate.architecture.clone())
            .merge_default(
                manifest
                    .autoupdate
                    .as_ref()
                    .unwrap()
                    .autoupdate_config
                    .clone(),
            );

        let hash_extraction = autoupdate_config
            .hash
            .as_ref()
            .ok_or(HashError::MissingHashExtraction)?
            .as_object()
            .ok_or(HashError::HashExtractionUrl)?;

        let (url, submap) = {
            let url = manifest
                .architecture
                .merge_default(manifest.install_config.clone())
                .url
                .as_ref()
                .ok_or(HashError::UrlNotFound)
                .and_then(|url| Ok(Url::parse(url)?))?;

            let mut submap = SubstitutionMap::new();
            submap.append_version(&manifest.version);
            submap.append_url(&url);

            let url = hash_extraction
                .url
                .as_ref()
                .ok_or(HashError::UrlNotFound)
                .map(|url| url.clone().into_substituted(&submap, false))
                .and_then(|url: String| Ok(Url::parse(&url)?))?;

            (url, submap)
        };

        let hash_mode = HashMode::from_autoupdate_config(&autoupdate_config)
            .ok_or(HashError::MissingHashExtraction)?;
        let source = reqwest::blocking::get(url.as_str())?;

        if hash_mode == HashMode::HashUrl {
            let hash = reqwest::blocking::get(url)?.text()?;

            return Ok(Hash {
                hash,
                hash_type: HashType::default(),
            });
        }

        if hash_mode == HashMode::Download {
            todo!("Download and compute hashes")
        }

        let hash = match hash_mode {
            HashMode::Extract(regex) => Hash::from_text(source.text()?, &submap, regex),
            HashMode::Xpath(xpath) => Hash::find_hash_in_xml(source.text()?, &submap, xpath),
            HashMode::Json(json_path) => Hash::from_json(source.bytes()?, &submap, json_path),
            HashMode::Rdf => Hash::from_rdf(source.bytes()?, url.remote_filename()),
            // HashMode::Fosshub => todo!(),
            // HashMode::Sourceforge => todo!(),
            _ => unreachable!(),
        }?;

        Ok(hash)
    }

    /// Compute a hash from a source
    pub fn compute(reader: impl BufRead, hash_type: HashType) -> Hash {
        use digest::Digest;

        fn compute_hash<D: Digest>(mut reader: impl BufRead) -> Vec<u8> {
            let mut hasher = D::new();

            loop {
                let bytes = reader.fill_buf().unwrap();
                if bytes.is_empty() {
                    break;
                }

                hasher.update(bytes);

                let len = bytes.len();
                reader.consume(len);
            }

            hasher.finalize()[..].to_vec()
        }

        let hash_bytes = match hash_type {
            HashType::Sha512 => compute_hash::<sha2::Sha512>(reader),
            HashType::Sha256 => compute_hash::<sha2::Sha256>(reader),
            HashType::Sha1 => compute_hash::<sha1::Sha1>(reader),
            HashType::MD5 => compute_hash::<md5::Md5>(reader),
        };

        let mut hash = String::new();
        for byte in hash_bytes {
            hash += &format!("{byte:02x}");
        }

        Hash { hash, hash_type }
    }

    /// Parse a hash from an RDF source
    ///
    /// # Errors
    /// - If the hash is not found
    pub fn from_rdf(source: impl AsRef<[u8]>, file_name: impl AsRef<str>) -> Result<Hash> {
        Ok(formats::rdf::parse_xml(source, file_name).map(|hash| {
            let hash_type = HashType::try_from(&hash).unwrap_or_default();
            Hash { hash, hash_type }
        })?)
    }

    /// Parse a hash from a text source
    ///
    /// # Errors
    /// - If the hash is not found
    /// - If the hash is invalid
    pub fn from_text(
        source: impl AsRef<str>,
        substitutions: &SubstitutionMap,
        regex: impl AsRef<str>,
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
        substitutions: &SubstitutionMap,
        json_path: impl AsRef<str>,
    ) -> Result<Hash> {
        let json = serde_json::from_slice(source.as_ref())?;

        let hash = formats::json::parse_json(&json, substitutions, json_path)?;
        let hash_type = HashType::try_from(&hash)?;

        Ok(Hash { hash, hash_type })
    }

    /// Parse a hash from an XML source
    ///
    /// # Errors
    /// - If the hash is not found
    /// - If the hash is invalid
    /// - If the XML is invalid
    /// - If the `XPath` is invalid
    pub fn find_hash_in_xml(
        source: impl AsRef<str>,
        substitutions: &SubstitutionMap,
        xpath: impl AsRef<str>,
    ) -> Result<Hash> {
        let hash = formats::xml::parse_xml(source, substitutions, xpath)?;
        let hash_type = HashType::try_from(&hash)?;

        Ok(Hash { hash, hash_type })
    }

    /// Find a hash in the headers of a response
    ///
    /// # Errors
    /// peepeepoopoo
    pub fn find_hash_in_headers(_headers: &HeaderMap<HeaderValue>) -> Result<Hash> {
        unimplemented!("I can't find a location where this is ever used")
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::{
        buckets::Bucket,
        packages::manifest::{HashExtractionOrArrayOfHashExtractions, StringArray},
    };

    use super::*;

    #[test]
    fn test_compute_hashes() {
        let data = b"hello world";

        let md5 = Hash::compute(BufReader::new(&data[..]), HashType::MD5);
        assert_eq!(md5.hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");

        let sha1 = Hash::compute(BufReader::new(&data[..]), HashType::Sha1);
        assert_eq!(sha1.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");

        let sha256 = Hash::compute(BufReader::new(&data[..]), HashType::Sha256);
        assert_eq!(
            sha256.hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        let sha512 = Hash::compute(BufReader::new(&data[..]), HashType::Sha512);
        assert_eq!(
            sha512.hash,
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
    }

    #[test]
    fn test_google_chrome_hashes() {
        let manifest = Bucket::from_name("extras")
            .unwrap()
            .get_manifest("googlechrome")
            .unwrap();

        let autoupdate = manifest
            .autoupdate
            .unwrap()
            .architecture
            .unwrap()
            .x64
            .unwrap();

        let HashExtractionOrArrayOfHashExtractions::HashExtraction(x64_cfg) =
            autoupdate.hash.unwrap()
        else {
            unreachable!()
        };

        let url = x64_cfg.url.unwrap().to_string();
        let xpath = x64_cfg.xpath.unwrap().to_string();

        let source = reqwest::blocking::get(url).unwrap().text().unwrap();

        let Some(url) = autoupdate.url else {
            unreachable!()
        };

        let url = Url::parse(&url).unwrap();

        let mut submap = SubstitutionMap::new();
        submap.append_version(&manifest.version);
        submap.append_url(&url);

        let hash = Hash::find_hash_in_xml(source, &submap, xpath).unwrap();

        let StringArray::String(actual_hash) =
            manifest.architecture.unwrap().x64.unwrap().hash.unwrap()
        else {
            unreachable!();
        };

        assert_eq!(actual_hash, hash.hash);
    }

    #[ignore = "replaced"]
    #[test]
    fn test_get_hash_for_googlechrome() {
        let manifest = Bucket::from_name("extras")
            .unwrap()
            .get_manifest("googlechrome")
            .unwrap();

        let hash = Hash::get_for_app(&manifest).unwrap();

        let actual_hash = manifest
            .architecture
            .unwrap()
            .x64
            .unwrap()
            .hash
            .unwrap()
            .to_string();

        assert_eq!(actual_hash, hash.hash);
    }
}
