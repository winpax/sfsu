use std::{fs::File, io::Write, path::PathBuf};

use itertools::Itertools;

use crate::{
    packages::{downloading::DownloadUrl, Manifest},
    Scoop, SupportedArch,
};

#[derive(Debug)]
pub struct ScoopCache;

impl ScoopCache {
    #[must_use]
    pub fn open_manifest(
        manifest: &Manifest,
        arch: Option<SupportedArch>,
    ) -> Option<std::io::Result<Vec<ScoopCacheWriter>>> {
        let name = &manifest.name;
        let version = &manifest.version;

        let urls = manifest.download_urls(arch.unwrap_or_default())?;

        let safe_urls = urls.into_iter().map(|url| {
            let safe_url = PathBuf::from(&url);
            (url, safe_url)
        });

        Some(
            safe_urls
                .map(|(url, file_name)| {
                    (url, format!("{}#{}#{}", name, version, file_name.display()))
                })
                .map(|(url, file_name)| (url, PathBuf::from(file_name)))
                .map(|(url, file_name)| ScoopCacheWriter::new(file_name, url.url))
                .collect(),
        )
    }
}

#[derive(Debug)]
pub struct ScoopCacheWriter {
    url: String,
    pub file_name: PathBuf,
    fp: File,
}

impl ScoopCacheWriter {
    pub fn new(file_name: PathBuf, url: String) -> std::io::Result<Self> {
        Ok(Self {
            fp: File::create(Scoop::cache_path().join(&file_name))?,
            url,
            file_name,
        })
    }
}

impl Write for ScoopCacheWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.fp.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.fp.flush()
    }
}
