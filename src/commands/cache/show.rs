
use clap::Parser;
use sprinkles::{
    abandon,
    output::structured::Structured,
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
