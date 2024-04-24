//! Cache helpers

use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    ops::Deref,
    path::{Path, PathBuf},
};

use digest::Digest;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{blocking::Response, StatusCode};

use crate::{
    hash::{url_ext::UrlExt, HashType},
    packages::Manifest,
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
}

#[derive(Debug)]
/// A cache handle
pub struct Handle {
    url: String,
    /// The file name
    pub file_name: PathBuf,
    hash_type: HashType,
    fp: File,
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
    ) -> std::io::Result<Self> {
        let file_name = file_name.into();
        Ok(Self {
            fp: File::create(cache_path.as_ref().join(&file_name))?,
            url,
            hash_type,
            file_name,
        })
    }

    #[must_use]
    /// Open a manifest and return a list of cache handles
    pub fn open_manifest(
        cache_path: impl AsRef<Path>,
        manifest: &Manifest,
    ) -> Option<std::io::Result<Self>> {
        let name = &manifest.name;
        let version = &manifest.version;

        let url = manifest.download_url()?;

        let file_name = PathBuf::from(&url);
        let file_name = format!("{}#{}#{}", name, version, file_name.display());

        Some(Self::new(
            cache_path.as_ref(),
            PathBuf::from(file_name),
            HashType::default(),
            url.url,
        ))
    }

    /// Create a new downloader
    ///
    /// # Errors
    /// - If the request fails
    pub fn begin_download(
        self,
        client: &impl Deref<Target = reqwest::blocking::Client>,
        mp: Option<&MultiProgress>,
    ) -> Result<Downloader, Error> {
        Downloader::new(self, client, mp)
    }
}

impl Write for Handle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.fp.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.fp.flush()
    }
}

#[derive(Debug)]
#[must_use = "Does nothing until `download` is called"]
/// A cache handle downloader
pub struct Downloader {
    cache: Handle,
    resp: Response,
    pb: Option<ProgressBar>,
    message_ptr: &'static str,
}

impl Downloader {
    /// Create a new downloader
    ///
    /// # Errors
    /// - If the request fails
    ///
    /// # Panics
    /// - A non-empty file name
    /// - Invalid progress style template
    pub fn new(
        cache: Handle,
        client: &impl Deref<Target = reqwest::blocking::Client>,
        mp: Option<&MultiProgress>,
    ) -> Result<Self, Error> {
        let resp = client.get(&cache.url).send()?;

        if !resp.status().is_success() {
            return Err(Error::ErrorCode(resp.status()));
        }

        debug!("Status Code: {}", resp.status());

        let content_length = resp.content_length().unwrap_or_default();

        // TODO: Implement a way to free this later to avoid (negligable) memory leaks
        let boxed = {
            if let Ok(parsed_url) = url::Url::parse(&cache.url)
                && let Some(leaf) = parsed_url.leaf()
            {
                leaf.into_boxed_str()
            } else {
                cache
                    .file_name
                    .to_string_lossy()
                    .split('_')
                    .next_back()
                    .expect("non-empty file name")
                    .to_string()
                    .into_boxed_str()
            }
        };

        let message: &'static str = Box::leak(boxed);

        let pb = mp.map(|mp| {
            mp.add(
            ProgressBar::new(content_length)
                .with_style(
                    ProgressStyle::with_template(
                        "{msg} {spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} {bytes_per_sec} ({eta})",
                    )
                    .unwrap()
                    .progress_chars("#>-"),
                )
                .with_message(message)
                .with_finish(indicatif::ProgressFinish::WithMessage("Finished ✅".into())),
        )
        });

        Ok(Self {
            cache,
            resp,
            pb,
            message_ptr: message,
        })
    }

    /// Download the file to the cache
    ///
    /// Returns the cache file name, and the computed hash
    ///
    /// # Errors
    /// - If the file cannot be written to the cache
    pub fn download(mut self) -> Result<(PathBuf, Vec<u8>), Error> {
        let hash_bytes = match self.cache.hash_type {
            HashType::SHA512 => self.handle_buf::<sha2::Sha512>(),
            HashType::SHA256 => self.handle_buf::<sha2::Sha256>(),
            HashType::SHA1 => self.handle_buf::<sha1::Sha1>(),
            HashType::MD5 => self.handle_buf::<md5::Md5>(),
        }?;

        // Hash;

        // loop {
        //     let chunk = reader.fill_buf()?;
        //     let chunk_length = chunk.len();

        //     if chunk_length == 0 {
        //         break;
        //     }

        //     self.cache.write_all(chunk)?;
        //     self.pb.inc(chunk_length as u64);

        //     reader.consume(chunk_length);
        // }

        Ok((self.cache.file_name.clone(), hash_bytes))
    }

    fn handle_buf<D: Digest>(&mut self) -> Result<Vec<u8>, Error> {
        let mut reader = BufReader::new(self.resp.by_ref());
        let mut hasher = D::new();

        loop {
            let chunk = reader.fill_buf().unwrap();
            if chunk.is_empty() {
                break;
            }

            hasher.update(chunk);
            self.cache.write_all(chunk)?;

            let chunk_length = chunk.len();

            if let Some(pb) = &self.pb {
                pb.inc(chunk_length as u64);
            }

            reader.consume(chunk_length);
        }

        Ok(hasher.finalize()[..].to_vec())
    }
}

impl Drop for Downloader {
    fn drop(&mut self) {
        // There is no code that would drop this message
        // As such this should be safe
        drop(unsafe { Box::from_raw(std::ptr::from_ref::<str>(self.message_ptr).cast_mut()) });
    }
}
