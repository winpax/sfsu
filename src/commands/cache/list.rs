use clap::Parser;
use itertools::Itertools;
use crate::contexts::ScoopContext;

use crate::{
    commands::Command,
    output::{colours::eprintln_bright_yellow, structured::Structured},
    wrappers::sizes::Size,
};

use super::CacheEntry;

#[derive(Debug, Clone, Parser)]
/// List cache entries
pub struct Args {
    #[clap(from_global)]
    pub(super) apps: Vec<String>,

    #[clap(from_global)]
    pub(super) glob: bool,

    #[clap(from_global)]
    pub(super) json: bool,
}

impl Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let cache_entries = CacheEntry::match_paths(ctx, &self.apps, self.glob).await?;

        let total_size = cache_entries
            .iter()
            .fold(Size::new(0), |acc, entry| acc + entry.size());

        eprintln_bright_yellow!("Total: {} files, {total_size}", cache_entries.len());

        let values = cache_entries
            .into_iter()
            .map(|entry| match entry {
                CacheEntry::Known {
                    name,
                    version,
                    size,
                    hash: url,
                    ..
                } => DisplayCacheEntry {
                    name,
                    version,
                    size,
                    url,
                },
                CacheEntry::Loose { file_path, size } => DisplayCacheEntry {
                    name: file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    version: "N/A".to_string(),
                    size,
                    url: "N/A".to_string(),
                },
            })
            .sorted_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).reverse())
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;

        // TODO: Figure out max length so urls aren't truncated unless they need to be
        let data = Structured::new(&values);

        println!("{data}");

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
struct DisplayCacheEntry {
    name: String,
    version: String,
    size: Size,
    url: String,
}
