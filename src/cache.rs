use std::{
    fs::File,
    io::{Read, Write},
    ops::Deref,
    path::PathBuf,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Response;

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
    pub fn begin_download(
        self,
        client: &impl Deref<Target = reqwest::blocking::Client>,
        mp: &MultiProgress,
    ) -> reqwest::Result<Downloader> {
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
pub struct Downloader {
    cache: Handle,
    resp: Response,
    pb: ProgressBar,
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
        mp: &MultiProgress,
    ) -> reqwest::Result<Self> {
        let url = cache.url.clone();
        let resp = client.get(url).send()?;

        let content_length = resp.content_length().unwrap_or_default();

        // TODO: Implement a way to free this later to avoid (negligable) memory leaks
        let boxed = cache
            .file_name
            .to_string_lossy()
            .to_string()
            .split('_')
            .next_back()
            .expect("non-empty file name")
            .to_string()
            .into_boxed_str();

        let message: &'static str = Box::leak(boxed);

        let pb = mp.add(
            ProgressBar::new(content_length)
                .with_style(
                    ProgressStyle::with_template(
                        "{msg} {spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                    )
                    .unwrap()
                    .progress_chars("#>-"),
                )
                .with_message(message)
                .with_finish(indicatif::ProgressFinish::WithMessage("Finished ✅".into())),
        );

        Ok(Self {
            cache,
            resp,
            pb,
            message_ptr: message,
        })
    }

    /// Download the file to the cache
    ///
    /// # Errors
    /// - If the file cannot be written to the cache
    ///
    /// # Panics
    /// - The request did not provide a content length
    pub fn download(mut self) -> std::io::Result<()> {
        let total_length = self.resp.content_length().expect("content length");
        let mut current = 0;

        let mut chunk = [0; 1024];

        loop {
            // Ensures that `read_exact` does not exhaust the reader, and throw an error in the final chunk
            if total_length - current < 1024 {
                break;
            }

            self.read_exact(&mut chunk)?;

            self.cache.write_all(&chunk)?;
            current += 1024;
        }

        // Handles all remaning data
        let mut final_chunk = vec![];
        self.read_to_end(&mut final_chunk)?;
        self.cache.write_all(&final_chunk)?;

        Ok(())
    }
}

impl Read for Downloader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let read = self.resp.read(buf)?;
        self.pb.inc(read as u64);
        Ok(read)
    }
}

impl Drop for Downloader {
    fn drop(&mut self) {
        // There is no code that would drop this message
        // As such this should be safe
        drop(unsafe { Box::from_raw((self.message_ptr as *const str).cast_mut()) });
    }
}
