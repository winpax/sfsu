use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use indicatif::{MultiProgress, ProgressBar};
use itertools::Itertools;
use reqwest::blocking::{Client, Response};

use crate::{
    packages::{downloading::DownloadUrl, Manifest},
    Scoop, SupportedArch,
};

#[derive(Debug)]
pub struct ScoopCache {
    url: String,
    pub file_name: PathBuf,
    fp: File,
}

impl ScoopCache {
    pub fn new(file_name: PathBuf, url: String) -> std::io::Result<Self> {
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

    pub fn begin_download(self, client: &Client) {
        // let res =
        todo!()
    }
}

impl Write for ScoopCache {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.fp.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.fp.flush()
    }
}

#[derive(Debug)]
pub struct CacheDownloader {
    cache: ScoopCache,
    resp: Response,
    pb: ProgressBar,
}

impl CacheDownloader {
    pub fn new(cache: ScoopCache, client: &Client, mp: &MultiProgress) -> reqwest::Result<Self> {
        let url = cache.url.clone();
        let resp = client.get(url).send()?;

        let content_length = resp.content_length().unwrap_or_default();

        // TODO: Implement a way to free this later to avoid (negligable) memory leaks
        let boxed = cache
            .file_name
            .to_string_lossy()
            .to_string()
            .into_boxed_str();

        let pb =
            mp.add(ProgressBar::new(content_length).with_message(Box::leak(boxed) as &'static str));

        Ok(Self { cache, resp, pb })
    }

    pub fn download(mut self) -> std::io::Result<()> {
        let mut buf = vec![];

        self.read_to_end(&mut buf)?;
        self.cache.write_all(&buf)?;

        Ok(())
    }
}

impl Read for CacheDownloader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.resp.read(buf)?;
        self.pb.inc(read as u64);
        Ok(read)
    }
}
