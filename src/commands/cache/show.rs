use clap::Parser;
use sprinkles::{
    abandon,
    output::{structured::Structured, wrappers::sizes::Size},
};

use crate::commands::{cache::CacheEntry, Command};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    pub apps: Vec<String>,

    #[clap(from_global)]
    pub json: bool,
}

impl Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let cache_entries = CacheEntry::match_paths(&self.apps).await?;

        if cache_entries.is_empty() {
            abandon!("No cache entries found");
        }

        let total_size = cache_entries
            .iter()
            .fold(Size::new(0), |acc, entry| acc + entry.size);

        warn!("Total: {} files, {total_size}", cache_entries.len());

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
