use std::{fs::File, io::Write, path::PathBuf};

use itertools::Itertools;
use reqwest::blocking::Client;

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

    pub fn download(self, client: &Client) {
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
