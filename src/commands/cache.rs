use std::os::windows::fs::MetadataExt;

use anyhow::Context;
use clap::Parser;
use serde::Serialize;
use sprinkles::{
    output::{structured::Structured, wrappers::sizes::Size},
    Scoop,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, PartialOrd)]
struct CacheEntry {
    name: String,
    version: String,
    size: Size,
    url: String,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let cache_path = Scoop::cache_path();

        let mut dir = tokio::fs::read_dir(cache_path).await?;

        let cache_entries =
            futures::future::try_join_all(dir.next_entry().await?.iter().map(|entry| async {
                let metadata = entry.metadata().await?;

                let cache_entry = {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    let mut parts = name.split('#');

                    let name = parts.next().context("No name")?;
                    let version = parts.next().context("No version")?;
                    let url = parts.next().context("No url")?;

                    #[allow(clippy::cast_precision_loss)]
                    let size = Size::new(metadata.file_size() as f64);

                    CacheEntry {
                        name: name.to_string(),
                        version: version.to_string(),
                        url: url.to_string(),
                        size,
                    }
                };

                anyhow::Ok(serde_json::to_value(cache_entry)?)
            }))
            .await?;

        // TODO: Figure out max length so urls aren't truncated unless they need to be
        let data = Structured::new(&cache_entries).with_max_length(50);

        println!("{data}");

        Ok(())

        // todo!()
    }
}
