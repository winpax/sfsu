//! Cache helpers

use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use bytes::BytesMut;
use digest::Digest;
use futures::{Stream, StreamExt, TryStreamExt};
use indicatif::{MultiProgress, ProgressBar};
use reqwest::{Response, StatusCode};

use crate::packages::downloading::Downloader;
use crate::{
    Architecture,
    hash::{Hash, HashType, url_ext::UrlExt},
    packages::{Manifest, downloading::DownloadUrl, models::manifest::SingleOrArray},
    progress,
    requests::ClientLike,
    version::Version,
};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Cache error
pub enum Error {
    #[error("Failed to download file: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Failed to write to file: {0}")]
    IO(#[from] std::io::Error),
    #[error("HTTP Error: {0}")]
    ErrorCode(StatusCode),
    #[error("Missing download url in manifest")]
    MissingDownloadUrl,
    #[error("Non-utf8 file name")]
    InvalidFileName,
    #[error("Missing parts in output file name")]
    MissingParts,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// A cache file handle
pub struct CacheFile<'a> {
    name: &'a str,
    version: &'a Version,
    url: &'a DownloadUrl,
}

impl<'a> CacheFile<'a> {
    #[must_use]
    /// Create a new cache file handle
    pub fn new(name: &'a str, version: &'a Version, url: &'a DownloadUrl) -> Self {
        Self { name, version, url }
    }

    #[must_use]
    #[deprecated]
    /// Get the cache file name using the legacy format
    pub fn filename_legacy(&self) -> String {
        let mut file_name = self.name.to_string();
        file_name += "#";
        file_name += self.version.as_str();
        file_name += "#";

        let url_path = {
            let mut url_path = PathBuf::from(self.url);

            if let Some(file_name) = self.url.file_name.as_ref() {
                let file_name = PathBuf::from(file_name);
                if let Some(extension) = file_name.extension().map(ToOwned::to_owned) {
                    url_path.set_extension(extension);
                }
            }

            url_path
        };

        file_name += &url_path.display().to_string();

        file_name
    }

    #[must_use]
    /// Get the cache file name
    pub fn filename(&self) -> String {
        let sha256_hash = {
            use sha2::{Digest, Sha256};

            let mut hasher = Sha256::new();

            let url = self.url.full_url();

            hasher.update(url.as_bytes());

            &format!("{:x}", hasher.finalize())[0..7]
        };

        let extension = if let Some(dest_path) = self.url.file_name.as_ref() {
            PathBuf::from(dest_path)
        } else {
            PathBuf::from(self.url)
        }
        .extension()
        .map(ToOwned::to_owned);

        let mut file_name = String::new();

        file_name += self.name;
        file_name += "#";
        file_name += self.version.as_str();
        file_name += "#";
        file_name += sha256_hash;

        if let Some(extension) = extension {
            file_name += ".";
            file_name += &(extension.to_string_lossy());
        }

        file_name
    }
}

#[derive(Debug, Clone)]
/// The file name to download
pub struct DownloadFileName {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Escaped download url
    pub url: String,
}

impl Display for DownloadFileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}#{}", self.name, self.version, self.url)
    }
}

impl TryFrom<&Path> for DownloadFileName {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file_name = path
            .file_name()
            .ok_or(Error::InvalidFileName)?
            .to_string_lossy();
        let mut parts = file_name.split('#');

        let name = parts.next().ok_or(Error::MissingParts)?.to_string();
        let version = parts.next().ok_or(Error::MissingParts)?.to_string();
        let url = parts.next().ok_or(Error::MissingParts)?.to_string();

        Ok(Self { name, version, url })
    }
}

impl From<DownloadFileName> for PathBuf {
    fn from(name: DownloadFileName) -> Self {
        let file_name = name.to_string();
        PathBuf::from(file_name)
    }
}

#[derive(Debug)]
/// Result for downloading
pub struct DownloadResult {
    /// Output file name
    pub file_name: DownloadFileName,
    /// The computed hash
    pub computed_hash: Hash,
    /// The hash stored in the manifest
    pub actual_hash: Hash,
}

#[derive(Debug)]
/// A cache handle
pub struct Handle {
    url: String,
    /// The cache output file name
    pub file_name: PathBuf,
    /// The cache output file path
    cache_path: PathBuf,
    hash_type: HashType,
    actual_hash: Hash,
}

impl Handle {
    /// Construct a new cache handle
    ///
    /// # Errors
    /// - If the file cannot be created
    pub fn new(
        cache_path: impl AsRef<Path>,
        file_name: impl Into<PathBuf>,
        hash_type: HashType,
        url: String,
        actual_hash: Hash,
    ) -> Result<Self, Error> {
        let file_name = file_name.into();
        let cache_path = cache_path.as_ref().join(&file_name);
        Ok(Self {
            url,
            file_name,
            cache_path,
            hash_type,
            actual_hash,
        })
    }

    /// Open a manifest and return a cache handle
    ///
    /// # Errors
    /// - IO errors
    /// - Missing download URL
    pub fn open_manifest(
        cache_path: impl AsRef<Path>,
        manifest: &Manifest,
        arch: Architecture,
    ) -> Result<Vec<Self>, Error> {
        let name = unsafe { manifest.name() };
        let version = &manifest.version;

        let download_urls = manifest
            .download_urls(arch)
            .ok_or(Error::MissingDownloadUrl)?
            .into_iter();

        let hashes = manifest
            .install_config(arch)
            .hash
            .map(SingleOrArray::to_vec)
            // .map(|hash| hash.map(Hash::hash_type).to_vec())
            .unwrap_or_default()
            .into_iter();

        download_urls
            .zip(hashes)
            .map(|(url, hash)| {
                let file_name = CacheFile::new(name, version, &url).filename();

                Self::new(
                    cache_path.as_ref(),
                    PathBuf::from(file_name),
                    hash.hash_type(),
                    url.url,
                    hash,
                )
            })
            .collect()
    }

    /// Create a new downloader
    ///
    /// # Errors
    /// - If the request fails
    pub async fn begin_download<T: ClientLike<reqwest::Client>>(
        self,
        mp: Option<&MultiProgress>,
    ) -> Result<DownloadHandle, Error> {
        DownloadHandle::new::<T>(self, mp, None).await
    }
}

#[derive(Debug)]
#[must_use = "Does nothing until `download` is called"]
/// A cache handle downloader
pub struct DownloadHandle {
    cache: Handle,
    resp: Response,
    pb: Option<ProgressBar>,
}

impl DownloadHandle {
    /// Create a new downloader
    ///
    /// # Errors
    /// - If the request fails
    ///
    /// # Panics
    /// - A non-empty file name
    /// - Invalid progress style template
    pub async fn new<T: ClientLike<reqwest::Client>>(
        cache: Handle,
        mp: Option<&MultiProgress>,
        name: Option<String>,
    ) -> Result<Self, Error> {
        let resp = T::new().client().get(&cache.url).send().await?;

        if !resp.status().is_success() {
            return Err(Error::ErrorCode(resp.status()));
        }

        debug!("Status Code: {}", resp.status());

        let content_length = resp.content_length().unwrap_or_default();

        let pb = mp.map(|mp| {
            let message = name.unwrap_or_else(|| {
                if let Ok(parsed_url) = url::Url::parse(&cache.url)
                    && let Some(leaf) = parsed_url.leaf()
                {
                    leaf.to_string()
                } else {
                    cache
                        .file_name
                        .to_string_lossy()
                        .split('_')
                        .next_back()
                        .expect("non-empty file name")
                        .to_string()
                }
            });

            mp.add(
                ProgressBar::new(content_length)
                    .with_style(progress::style(
                        Some(progress::ProgressOptions::Bytes),
                        Some(progress::Message::prefix().with_message(&message)),
                    ))
                    .with_finish(indicatif::ProgressFinish::WithMessage("Finished ✅".into())),
            )
        });

        Ok(Self { cache, resp, pb })
    }

    #[allow(tail_expr_drop_order)]
    async fn handle_buf<D: Digest>(self) -> Result<Vec<u8>, Error> {
        use tokio::{fs::File, io::AsyncWriteExt};
        use tokio_util::codec::{BytesCodec, FramedRead};

        enum Source<T: futures::Stream<Item = reqwest::Result<bytes::Bytes>> + std::marker::Unpin> {
            Cache(futures::prelude::stream::IntoStream<FramedRead<File, BytesCodec>>),
            Network(T),
        }

        impl<T> Stream for Source<T>
        where
            T: futures::Stream<Item = reqwest::Result<bytes::Bytes>> + std::marker::Unpin,
        {
            type Item = reqwest::Result<bytes::Bytes>;

            fn poll_next(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                match self.get_mut() {
                    Source::Cache(file) => file.poll_next_unpin(cx).map(|bytes| match bytes {
                        Some(Ok(bytes)) => Some(Ok(BytesMut::freeze(bytes))),
                        _ => None,
                    }),
                    Source::Network(resp) => resp.poll_next_unpin(cx),
                }
            }
        }

        let cache_path = self.cache.cache_path.clone();

        let mut reader = if cache_path.exists() {
            debug!("Loading from cache");
            if let Some(pb) = &self.pb {
                pb.set_prefix("📦");
            }
            let file = File::open(&cache_path).await?;
            let stream = FramedRead::with_capacity(file, BytesCodec::new(), {
                // 1 MiB buffer
                1024 * 1024
            });

            Source::Cache(stream.into_stream())
        } else {
            debug!("Downloading via network");
            Source::Network(self.resp.bytes_stream())
        };

        let mut cache_file = match &reader {
            Source::Cache(_) => None,
            Source::Network(_) => Some(File::create(&cache_path).await?),
        };

        let mut hasher = D::new();

        while let Some(Ok(chunk)) = reader.next().await {
            hasher.update(&chunk);

            if let Some(cache_file) = cache_file.as_mut() {
                cache_file.write_all(&chunk).await?;
            }

            let chunk_length = chunk.len();

            #[allow(if_let_rescope)]
            if let Some(pb) = self.pb.as_ref() {
                pb.inc(chunk_length as u64);
            }
        }

        Ok(hasher.finalize()[..].to_vec())
    }
}

impl Downloader for DownloadHandle {
    type Error = Error;
    type Output = DownloadResult;

    /// Download the file to the cache
    ///
    /// Returns the cache file name, and the computed hash
    ///
    /// # Errors
    /// - If the file cannot be written to the cache
    async fn download(self) -> Result<DownloadResult, Error> {
        let actual_hash = self.cache.actual_hash.clone();

        let file_name = self.cache.file_name.clone();
        let hash_bytes = match self.cache.hash_type {
            HashType::SHA512 => self.handle_buf::<sha2::Sha512>().await,
            HashType::SHA256 => self.handle_buf::<sha2::Sha256>().await,
            HashType::SHA1 => self.handle_buf::<sha1::Sha1>().await,
            HashType::MD5 => self.handle_buf::<md5::Md5>().await,
        }?;

        Ok(DownloadResult {
            file_name: file_name.as_path().try_into()?,
            computed_hash: Hash::from_hex(&hash_bytes),
            actual_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{cache::CacheFile, version::Version};

    #[test]
    fn test_cache_file_sfsu_1_13_4() {
        use crate::packages::downloading::DownloadUrl;

        let url = DownloadUrl::from_string(
            "https://github.com/jewlexx/sfsu/releases/download/v1.13.4/sfsu-x86_64.exe#/sfsu.exe",
        );

        let version = Version::new("1.13.4");
        let file = CacheFile::new("sfsu", &version, &url);

        assert_eq!(file.filename(), "sfsu#1.13.4#659892c.exe");
    }

    #[test]
    fn test_cache_file_sfsu_1_13_3() {
        use crate::packages::downloading::DownloadUrl;

        let url = DownloadUrl::from_string(
            "https://github.com/jewlexx/sfsu/releases/download/v1.13.3/sfsu-x86_64.exe#/sfsu.exe",
        );

        let version = Version::new("1.13.3");
        let file = CacheFile::new("sfsu", &version, &url);

        assert_eq!(file.filename(), "sfsu#1.13.3#26ae19b.exe");
    }

    #[test]
    #[allow(deprecated)]
    fn test_cache_file_legacy() {
        use crate::packages::downloading::DownloadUrl;

        let url = DownloadUrl::from_string(
            "https://github.com/jewlexx/sfsu/releases/download/v1.13.3/sfsu-x86_64.exe#/sfsu.exe",
        );

        let version = Version::new("1.13.3");
        let file = CacheFile::new("sfsu", &version, &url);

        assert_eq!(
            file.filename_legacy(),
            "sfsu#1.13.3#https_github.com_jewlexx_sfsu_releases_download_v1.13.3_sfsu-x86_64.exe_sfsu.exe"
        );
    }

    #[test]
    fn test_cache_file_renamed() {
        use crate::packages::downloading::DownloadUrl;

        let url = DownloadUrl::from_string(
            "https://archive.mozilla.org/pub/thunderbird/releases/115.11.1/win64/en-US/Thunderbird%20Setup%20115.11.1.exe#/dl.7z",
        );

        let version = Version::new("115.11.1");
        let file = CacheFile::new("thunderbird", &version, &url);

        assert_eq!(file.filename(), "thunderbird#115.11.1#c9628b6.7z");
    }

    #[test]
    #[allow(deprecated)]
    fn test_cache_file_renamed_legacy() {
        use crate::packages::downloading::DownloadUrl;

        let url = DownloadUrl::from_string(
            "https://archive.mozilla.org/pub/thunderbird/releases/115.11.1/win64/en-US/Thunderbird%20Setup%20115.11.1.exe#/dl.7z",
        );

        let version = Version::new("115.11.1");
        let file = CacheFile::new("thunderbird", &version, &url);

        assert_eq!(
            file.filename_legacy(),
            "thunderbird#115.11.1#https_archive.mozilla.org_pub_thunderbird_releases_115.11.1_win64_en-US_Thunderbird_20Setup_20115.11.1.exe_dl.7z"
        );
    }
}
