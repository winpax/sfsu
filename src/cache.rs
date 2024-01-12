use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use futures_util::StreamExt as _;
use indicatif::{MultiProgress, ProgressBar};
use reqwest::{Client, Response};
use tokio::runtime::Runtime;

use crate::{packages::Manifest, Scoop, SupportedArch};

#[derive(Debug)]
pub struct Handle {
    url: String,
    pub file_name: PathBuf,
    fp: File,
}

impl Handle {
    /// Construct a new cache handle
    ///
    /// # Errors
    /// - If the file cannot be created
    pub fn new(file_name: impl Into<PathBuf>, url: String) -> std::io::Result<Self> {
        let file_name = file_name.into();
        Ok(Self {
            fp: File::create(Scoop::cache_path().join(&file_name))?,
            url,
            file_name,
        })
    }

    #[must_use]
    pub fn open_manifest(
        manifest: &Manifest,
        arch: Option<SupportedArch>,
    ) -> Option<std::io::Result<Vec<Self>>> {
        let name = &manifest.name;
        let version = &manifest.version;

        let urls = manifest.download_urls(arch.unwrap_or_default())?;

        Some(
            urls.into_iter()
                .map(|url| {
                    let file_name = PathBuf::from(&url);
                    (url, format!("{}#{}#{}", name, version, file_name.display()))
                })
                .map(|(url, file_name)| Self::new(PathBuf::from(file_name), url.url))
                .collect(),
        )
    }

    /// Create a new downloader
    ///
    /// # Errors
    /// - If the request fails
    pub async fn begin_download(
        self,
        client: &Client,
        mp: &MultiProgress,
    ) -> reqwest::Result<Downloader> {
        Downloader::new(self, client, mp).await
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
pub struct Downloader {
    cache: Handle,
    resp: Response,
    pb: ProgressBar,
    message_ptr: *mut str,
}

impl Downloader {
    /// Create a new downloader
    ///
    /// # Errors
    /// - If the request fails
    pub async fn new(cache: Handle, client: &Client, mp: &MultiProgress) -> reqwest::Result<Self> {
        let url = cache.url.clone();
        let resp = client.get(url).send().await?;

        let content_length = resp.content_length().unwrap_or_default();

        let boxed = cache
            .file_name
            .to_string_lossy()
            .to_string()
            .into_boxed_str();

        let message: &'static str = Box::leak(boxed);

        let pb = mp.add(ProgressBar::new(content_length).with_message(message));

        Ok(Self {
            cache,
            resp,
            pb,
            message_ptr: (message as *const str).cast_mut(),
        })
    }

    /// Download the file to the cache
    ///
    /// # Errors
    /// - If the file cannot be written to the cache
    pub async fn download(mut self) -> anyhow::Result<()> {
        let mut bytes_stream = self.resp.bytes_stream();

        while let Some(chunk) = bytes_stream.next().await {
            let chunk = chunk?;

            self.pb.inc(chunk.len() as u64);
            self.cache.write_all(&chunk)?;
        }

        // while let Ok(read) = self.read_exact(&mut buf) {
        //     if read == 0 {
        //         break;
        //     }

        //     self.cache.write_all(&buf)?;
        // }

        // There is no code that would drop this message
        // As such this should be safe
        drop(unsafe { Box::from_raw(self.message_ptr) });

        Ok(())
    }
}
