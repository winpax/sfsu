use std::os::windows::fs::MetadataExt;

use anyhow::Context;
use clap::Parser;
use serde::Serialize;
use sprinkles::{output::wrappers::sizes::Size, Scoop};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

#[derive(Debug, Clone, Serialize)]
struct CacheEntry {
    name: String,
    version: String,
    url: String,
    size: Size,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let cache_path = Scoop::cache_path();

        let mut dir = tokio::fs::read_dir(cache_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;

            let cache_entry = {
                let name = entry.file_name();
                let mut parts = name.to_string_lossy().split('#');

                let name = parts.next().context("No name")?;
                let version = parts.next().context("No version")?;
                let url = parts.next().context("No url")?;
                let size = Size::new(metadata.file_size() as f64);

                CacheEntry {
                    name: name.to_string(),
                    version: version.to_string(),
                    url: url.to_string(),
                    size,
                }
            };

            #[allow(clippy::cast_precision_loss)]
            let size = Size::new(metadata.file_size() as f64);
            println!("{name}, {size} bytes");
        }

        Ok(())

        // todo!()
    }
}
