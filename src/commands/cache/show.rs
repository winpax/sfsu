use std::os::windows::fs::MetadataExt;

use anyhow::Context;
use clap::Parser;
use regex::Regex;
use serde::Serialize;
use sprinkles::{
    abandon,
    output::{structured::Structured, wrappers::sizes::Size},
    Scoop,
};
use tokio::task::JoinSet;

use crate::commands::Command;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct CacheEntry {
    name: String,
    version: String,
    size: Size,
    url: String,
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    pub apps: Vec<String>,

    #[clap(from_global)]
    pub json: bool,
}

impl Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let cache_path = Scoop::cache_path();

        let patterns = self
            .apps
            .into_iter()
            .filter_map(|pattern| Regex::new(&format!("^{pattern}#")).ok())
            .collect::<Vec<_>>();

        let mut set = JoinSet::new();
        let mut dir = tokio::fs::read_dir(cache_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if !patterns.iter().any(|pattern| pattern.is_match(&file_name)) {
                continue;
            }

            let file_name = file_name.to_string();

            set.spawn(async move {
                let metadata = entry.metadata().await?;

                let mut parts = file_name.split('#');

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

        if cache_entries.is_empty() {
            abandon!("No cache entries found");
        }

        cache_entries.sort();

        let values = cache_entries
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: Figure out max length so urls aren't truncated unless they need to be
        let data = Structured::new(&values).with_max_length(50);

        println!("{data}");

        Ok(())
    }
}
