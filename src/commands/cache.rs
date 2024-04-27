use std::os::windows::fs::MetadataExt;

use anyhow::Context;
use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;
use sprinkles::{
    output::{structured::Structured, wrappers::sizes::Size},
    Scoop,
};
use tokio::task::JoinSet;

use crate::commands::cache;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct CacheEntry {
    name: String,
    version: String,
    size: Size,
    url: String,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let cache_path = Scoop::cache_path();

        let mut set = JoinSet::new();
        let mut dir = tokio::fs::read_dir(cache_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            set.spawn(async move {
                let metadata = entry.metadata().await?;

                let name = entry.file_name();
                let name = name.to_string_lossy();
                let mut parts = name.split('#');

                let name = parts.next().context("No name")?;
                let version = parts.next().context("No version")?;
                let url = parts.next().context("No url")?;

                #[allow(clippy::cast_precision_loss)]
                let size = Size::new(metadata.file_size());

                let cache_entry = CacheEntry {
                    name: name.to_string(),
                    version: version.to_string(),
                    url: url.to_string(),
                    size,
                };

                anyhow::Ok(cache_entry)
            });
        }

        let mut cache_entries = {
            let mut cache_entries = vec![];

            while let Some(result) = set.join_next().await {
                let result = result??;
                cache_entries.push(result);
            }

            cache_entries
        };

        cache_entries.sort();

        let values = cache_entries
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: Figure out max length so urls aren't truncated unless they need to be
        let data = Structured::new(&values).with_max_length(50);

        println!("{data}");

        Ok(())

        // todo!()
    }
}
