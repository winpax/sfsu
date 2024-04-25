//! Manifest hashing utilities

use std::{io::BufRead, num::ParseIntError};

use formats::{json, text};
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use substitutions::SubstitutionMap;
use url::Url;

use crate::{
    cache::{self, Downloader, Handle},
    hash::url_ext::UrlExt,
    packages::{
        manifest::{
            AutoupdateConfig, HashExtractionOrArrayOfHashExtractions, HashMode as ManifestHashMode,
        },
        Manifest, MergeDefaults,
    },
    requests::AsyncClient,
    Architecture, Scoop,
};

use self::substitutions::Substitute;

pub(crate) mod formats;
pub(crate) mod substitutions;
pub(crate) mod url_ext;

/// Decode a hex string into bytes
///
/// # Errors
/// - Parse hex string to integer
/// - Convert chunk to utf8 string
pub fn decode_hex(hex: &str) -> Result<Vec<u8>, Error> {
    hex.as_bytes()
        .chunks(2)
        .map(|chunk| Ok(u8::from_str_radix(std::str::from_utf8(chunk)?, 16)?))
        .collect()
}

#[must_use]
/// Encode bytes into a hex string
pub fn encode_hex(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        result.push_str(&format!("{byte:02x}"));
    }
    result
}

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Hash errors
pub enum Error {
    #[error("Text error: {0}")]
    TextError(#[from] text::Error),
    #[error("Json error: {0}")]
    JsonError(#[from] json::Error),
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
    #[error("Missing autoupdate config")]
    MissingAutoupdateConfig,
    #[error("Cannot determine hash mode")]
    HashMode,
    #[error("Missing hash extraction object")]
    MissingHashExtraction,
    #[error("Hash extraction url where there should be a hash extraction object. This is a bug, please report it.")]
    HashExtractionUrl,
    #[error("Missing part of hash extraction object, where it should exist. This is a bug, please report it.")]
    MissingExtraction,
    #[error("Fosshub regex failed to match")]
    MissingFosshubCaptures,
    #[error("Sourceforge regex failed to match")]
    MissingSourceforgeCaptures,
    #[error("HTTP error: {0}")]
    ErrorStatus(StatusCode),
    #[error("Could not find a download url in the manifest for hash computation")]
    MissingDownloadUrl,
    #[error("Could not parse hex string as bytes")]
    DecodingHex(#[from] ParseIntError),
    #[error("Could not convert chunk to utf8 string")]
    DecodingHexUtf8(#[from] std::str::Utf8Error),
    #[error("Interacting with cache failed: {0}")]
    CacheError(#[from] cache::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Hash result type
pub struct Hash {
    hash: String,
    hash_type: HashType,
}

impl Hash {
    #[must_use]
    /// Get the hash string
    pub fn hash(&self) -> String {
        self.to_string()
    }

    #[must_use]
    /// Get the hash type
    pub fn hash_type(&self) -> HashType {
        self.hash_type
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.hash_type {
            HashType::SHA512 => "sha512:",
            HashType::SHA256 => "",
            HashType::SHA1 => "sha1:",
            HashType::MD5 => "md5:",
        };

        write!(f, "{prefix}")?;
        write!(f, "{}", self.hash)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
/// Hash types
pub enum HashType {
    /// SHA512
    SHA512,
    #[default]
    /// SHA256
    SHA256,
    /// SHA1
    SHA1,
    /// MD5
    MD5,
}

impl TryFrom<&String> for HashType {
    type Error = Error;

    fn try_from(value: &String) -> Result<Self, Error> {
        match value.len() {
            64 => Ok(HashType::SHA256),
            40 => Ok(HashType::SHA1),
            32 => Ok(HashType::MD5),
            128 => Ok(HashType::SHA512),
            _ => Err(Error::InvalidHash),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// A copy of the manifest `HashMode`, with associated data, and additional options
pub enum HashMode {
    /// Directly download the hash from the provided url
    HashUrl,
    #[default]
    /// Download the file and compute the hash
    Download,
    /// Extract the hash using the provided regex
    Extract(String),
    /// Extract the hash using the provided JSON path
    Json(String),
    /// Extract the hash using the provided `XPath`
    Xpath(String),
    /// Extract the hash from the Fosshub page
    Fosshub,
    /// Extract the hash from the Metalink (unimplemented)
    Metalink,
    /// Extract the hash from an RDF file
    Rdf,
    /// Extract the hash from the Sourceforge page
    Sourceforge,
}

impl HashMode {
    fn fosshub_regex() -> Regex {
        Regex::new(r"^(?:.*fosshub.com\/).*(?:\/|\?dwl=)(?<filename>.*)$")
            .expect("valid fosshub regex")
    }

    fn sourceforge_regex() -> Regex {
        Regex::new(r"(?:downloads\.)?sourceforge.net\/projects?\/(?<project>[^\/]+)\/(?:files\/)?(?<file>.*)").expect("valid sourceforge regex")
    }

    #[must_use]
    #[allow(deprecated)]
    /// Get a [`HashMode`] from an [`Manifest`]
    ///
    /// # Panics
    /// - Invalid regexes
    pub fn from_manifest(manifest: &Manifest, arch: Architecture) -> Option<Self> {
        let install_config = manifest
            .architecture
            .merge_default(manifest.install_config.clone(), arch);

        if let Some(url) = install_config.url {
            if Self::fosshub_regex().is_match(&url) {
                return Some(Self::Fosshub);
            }

            if Self::sourceforge_regex().is_match(&url) {
                return Some(Self::Sourceforge);
            }
        }

        let autoupdate_config = manifest
            .autoupdate
            .as_ref()
            .and_then(|autoupdate| autoupdate.architecture.clone())
            .merge_default(
                manifest.autoupdate.as_ref().unwrap().default_config.clone(),
                arch,
            );

        Self::from_autoupdate_config(&autoupdate_config)
    }

    #[must_use]
    #[deprecated(note = "Does not handle Sourceforge or Fosshub. Use `from_manifest` instead.")]
    /// Get a [`HashMode`] from an [`AutoupdateConfig`]
    pub fn from_autoupdate_config(config: &AutoupdateConfig) -> Option<Self> {
        let hash = config.hash.as_ref()?;

        match hash {
            HashExtractionOrArrayOfHashExtractions::Url(_) => Some(HashMode::Download),
            HashExtractionOrArrayOfHashExtractions::HashExtraction(hash_cfg) => {
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

                if let Some(mode) = mode {
                    Some(mode)
                } else {
                    Some(HashMode::HashUrl)
                }
            }
        }
    }
}

impl Hash {
    #[must_use]
    /// Create a new hash from hash hex bytes
    pub fn from_hex(bytes: &[u8]) -> Self {
        let hash = encode_hex(bytes);
        let hash_type = HashType::try_from(&hash).unwrap_or_default();

        Hash { hash, hash_type }
    }

    /// Get a hash for an app
    ///
    /// Note that this function expects the current url to be the updated version.
    /// In the case that we have to download and compute the hash, we will download the app from the current url, not the autoupdate url.
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
    pub async fn get_for_app(manifest: &Manifest, arch: Architecture) -> Result<Hash, Error> {
        let autoupdate_config = manifest
            .autoupdate_config(arch)
            .ok_or(Error::MissingAutoupdateConfig)?;

        let mut hash_mode = HashMode::from_manifest(manifest, arch).unwrap_or_default();

        if hash_mode == HashMode::Download {
            let cache_handle = Handle::open_manifest(Scoop::cache_path(), manifest, arch)?;

            let downloader = Downloader::new(cache_handle, &AsyncClient::new(), None).await?;

            let (_, hash) = downloader.download().await?;

            return Ok(Self::from_hex(&hash));

            // return if let Some(dl_url) = manifest
            //     .architecture
            //     .as_ref()
            //     .and_then(|arch| arch_field!(arch.url as ref).flatten())
            // {
            //     let source = BlockingClient::new().get(dl_url).send()?;
            //     Ok(Hash::compute(BufReader::new(source), HashType::SHA256))
            // } else {
            //     Err(HashError::MissingDownloadUrl)
            // };
        }

        let manifest_url = manifest
            .install_config(arch)
            .url
            .as_ref()
            .ok_or(Error::UrlNotFound)
            .and_then(|url| Ok(Url::parse(url)?))?;

        let submap = SubstitutionMap::from_all(&manifest.version, &manifest_url);

        let url = if matches!(hash_mode, HashMode::Fosshub | HashMode::Sourceforge) {
            let (url, regex): (Url, String) = match hash_mode {
                HashMode::Fosshub => {
                    let matches = HashMode::fosshub_regex()
                        .captures(manifest_url.as_str())
                        .ok_or(Error::MissingFosshubCaptures)?;

                    let regex = matches
                        .name("filename")
                        .ok_or(Error::MissingFosshubCaptures)?
                        .as_str()
                        .to_string()
                        + r#".*?"sha256":"([a-fA-F0-9]{64})""#;

                    // let source = BlockingClient::new().get(manifest_url).send()?.text()?;
                    // Hash::from_text(source, &SubstitutionMap::default(), regex);

                    (manifest_url, regex)
                }
                HashMode::Sourceforge => {
                    let matches = HashMode::sourceforge_regex()
                        .captures(manifest_url.as_str())
                        .ok_or(Error::MissingSourceforgeCaptures)?;

                    let project = matches
                        .name("project")
                        .ok_or(Error::MissingSourceforgeCaptures)?
                        .as_str();
                    let file = matches
                        .name("file")
                        .ok_or(Error::MissingSourceforgeCaptures)?
                        .as_str();

                    let hashfile_url = {
                        let url_string =
                            format!("https://sourceforge.net/projects/{project}/files/{file}");

                        let mut parsed_url = Url::parse(&url_string)?;
                        parsed_url.strip_fragment();
                        parsed_url.strip_filename();

                        parsed_url
                    };

                    let regex = r#""$basename":.*?"sha1":\s*"([a-fA-F0-9]{40})""#;

                    (hashfile_url, regex.to_string())
                }
                _ => unreachable!(),
            };

            hash_mode = HashMode::Extract(regex);

            url
        } else {
            let hash_extraction = autoupdate_config
                .hash
                .as_ref()
                .ok_or(Error::MissingHashExtraction)?
                .as_object()
                .ok_or(Error::HashExtractionUrl)?;

            hash_extraction
                .url
                .as_ref()
                .ok_or(Error::UrlNotFound)
                .map(|url| url.clone().into_substituted(&submap, false))
                .and_then(|url: String| Ok(Url::parse(&url)?))?
        };

        let source = AsyncClient::new().get(url.as_str()).send().await?;

        if hash_mode == HashMode::HashUrl {
            let hash = source.text().await?;
            let hash_type = HashType::try_from(&hash).unwrap_or_default();

            return Ok(Hash { hash, hash_type });
        }

        let hash = match hash_mode {
            HashMode::Extract(regex) => Hash::from_text(source.text().await?, &submap, regex),
            HashMode::Xpath(xpath) => Hash::find_hash_in_xml(source.text().await?, &submap, xpath),
            HashMode::Json(json_path) => Hash::from_json(source.bytes().await?, &submap, json_path),
            HashMode::Rdf => Hash::from_rdf(source.bytes().await?, url.remote_filename()),
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
            HashType::SHA512 => compute_hash::<sha2::Sha512>(reader),
            HashType::SHA256 => compute_hash::<sha2::Sha256>(reader),
            HashType::SHA1 => compute_hash::<sha1::Sha1>(reader),
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
    pub fn from_rdf(source: impl AsRef<[u8]>, file_name: impl AsRef<str>) -> Result<Hash, Error> {
        Ok(formats::rdf::parse_xml(source, file_name).map(|hash| {
            let hash_type = HashType::try_from(&hash).unwrap_or_default();
            Hash { hash, hash_type }
        })?)
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
    ) -> Result<Hash, Error> {
        let hash = formats::xml::parse_xml(source, substitutions, xpath)?;
        let hash_type = HashType::try_from(&hash)?;

        Ok(Hash { hash, hash_type })
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
    ) -> Result<Hash, Error> {
        let hash =
            formats::text::parse_text(source, substitutions, regex)?.ok_or(Error::NotFound)?;
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
    ) -> Result<Hash, Error> {
        let json = serde_json::from_slice(source.as_ref())?;

        let hash = formats::json::parse_json(&json, substitutions, json_path)?;
        let hash_type = HashType::try_from(&hash)?;

        Ok(Hash { hash, hash_type })
    }

    /// Find a hash in the headers of a response
    ///
    /// # Errors
    /// peepeepoopoo
    pub fn find_hash_in_headers(_headers: &HeaderMap<HeaderValue>) -> Result<Hash, Error> {
        unimplemented!("I can't find a location where this is ever used")
    }
}

#[cfg(test)]
mod tests {
    use std::{io::BufReader, str::FromStr};

    use crate::{
        buckets::Bucket,
        packages::{manifest::HashExtractionOrArrayOfHashExtractions, reference},
        requests::BlockingClient,
    };

    use super::*;

    #[test]
    fn test_compute_hashes() {
        let data = b"hello world";

        let md5 = Hash::compute(BufReader::new(&data[..]), HashType::MD5);
        assert_eq!(md5.hash, "5eb63bbbe01eeed093cb22bb8f5acdc3");

        let sha1 = Hash::compute(BufReader::new(&data[..]), HashType::SHA1);
        assert_eq!(sha1.hash, "2aae6c35c94fcfb415dbe95f408b9ce91ee846ed");

        let sha256 = Hash::compute(BufReader::new(&data[..]), HashType::SHA256);
        assert_eq!(
            sha256.hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        let sha512 = Hash::compute(BufReader::new(&data[..]), HashType::SHA512);
        assert_eq!(
            sha512.hash,
            "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
        );
    }

    #[test]
    #[ignore = "Broken (not my fault, the chrome xml file does not include the hash for the current version)"]
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

        let source = BlockingClient::new()
            .get(url)
            .send()
            .unwrap()
            .text()
            .unwrap();

        let Some(url) = autoupdate.url else {
            unreachable!()
        };

        let url = Url::parse(&url).unwrap();

        let mut submap = SubstitutionMap::new();
        submap.append_version(&manifest.version);
        submap.append_url(&url);

        let hash = Hash::find_hash_in_xml(source, &submap, xpath).unwrap();

        let actual_hash = manifest.architecture.unwrap().x64.unwrap().hash.unwrap();

        assert_eq!(actual_hash, hash.hash);
    }

    #[ignore = "replaced"]
    #[tokio::test]
    async fn test_get_hash_for_googlechrome() {
        let manifest = Bucket::from_name("extras")
            .unwrap()
            .get_manifest("googlechrome")
            .unwrap();

        let hash = Hash::get_for_app(&manifest, Architecture::ARCH)
            .await
            .unwrap();

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

    pub struct TestHandler {
        package: reference::Package,
    }

    // TODO: Implement tests for entire application autoupdate

    impl TestHandler {
        pub fn new(package: reference::Package) -> Self {
            Self { package }
        }

        pub async fn test(self) -> anyhow::Result<()> {
            let manifest = self.package.manifest().await?;

            let hash = Hash::get_for_app(&manifest, Architecture::ARCH).await?;

            let actual_hash = manifest
                .architecture
                .merge_default(manifest.install_config, Architecture::ARCH)
                .hash
                .unwrap();

            assert_eq!(actual_hash, hash.hash());

            Ok(())
        }
    }

    #[tokio::test]
    #[ignore = "Duplicate of `test_googlechrome`"]
    async fn test_handlers_implemented() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/googlechrome")?;

        let handler = TestHandler::new(package);

        handler.test().await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore = "Broken (not my fault, the chrome xml file does not include the hash for the current version)"]
    async fn test_googlechrome() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/googlechrome")?;

        let handler = TestHandler::new(package);

        handler.test().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_springboot() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/springboot")?;
        let handler = TestHandler::new(package);
        handler.test().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_sfsu() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/sfsu")?;

        let handler = TestHandler::new(package);

        handler.test().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_keepass() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/keepass")?;

        let handler = TestHandler::new(package);

        handler.test().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_hwinfo() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/hwinfo")?;

        let handler = TestHandler::new(package);

        handler.test().await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore = "sfsu does not yet support custom matches"]
    async fn test_ungoogled_chromium() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/ungoogled-chromium")?;
        let handler = TestHandler::new(package);
        handler.test().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_firefox() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/firefox")?;
        TestHandler::new(package).test().await
    }

    #[tokio::test]
    async fn test_sfsu_autoupdate() -> anyhow::Result<()> {
        let mut package = reference::Package::from_str("extras/sfsu")?;
        package.set_version("1.10.2".to_string());
        let manifest = package.manifest().await?;

        assert_eq!(
            manifest.architecture.unwrap().x64.unwrap().hash.unwrap(),
            "84344247cc06339d0dc675a49af81c37f9488e873e74e9701498d06f8e4db588"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_computed_hash() -> anyhow::Result<()> {
        let package = reference::Package::from_str("extras/sfsu")?;
        let mut manifest = package.manifest().await?;
        manifest.autoupdate.as_mut().unwrap().default_config.hash = None;

        manifest.set_version("1.10.2".to_string()).await?;

        assert_eq!(
            manifest.architecture.unwrap().x64.unwrap().hash.unwrap(),
            "84344247cc06339d0dc675a49af81c37f9488e873e74e9701498d06f8e4db588"
        );

        Ok(())
    }
}
