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
        &self,
        manifest: &Manifest,
        arch: Option<SupportedArch>,
    ) -> Option<std::io::Result<Vec<ScoopCacheWriter>>> {
        let name = &manifest.name;
        let version = &manifest.version;

        let urls = manifest.download_urls(arch.unwrap_or_default())?;

        let safe_urls = urls.into_iter().map(PathBuf::from);

        Some(
            safe_urls
                .map(|file_name| format!("{}#{}#{}", name, version, file_name.display()))
                .map(PathBuf::from)
                .map(ScoopCacheWriter::new)
                .collect(),
        )
    }
}

#[derive(Debug)]
pub struct ScoopCacheWriter {
    file_name: PathBuf,
    fp: File,
}

impl ScoopCacheWriter {
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        Ok(Self {
            fp: File::create(Scoop::cache_path().join(&path))?,
            file_name: path,
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
