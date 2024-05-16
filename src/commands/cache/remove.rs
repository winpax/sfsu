use clap::Parser;
use sprinkles::{config, contexts::ScoopContext, wrappers::sizes::Size};

use crate::{commands::Command, output::colours::eprintln_bright_yellow};

use super::CacheEntry;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    apps: Vec<String>,
}

impl Command for Args {
    async fn runner(self, ctx: impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        let cache_entries = CacheEntry::match_paths(&ctx, &self.apps).await?;

        let total_entires = cache_entries.len();
        let total_size = cache_entries
            .iter()
            .fold(Size::new(0), |acc, entry| acc + entry.size);

        let cache_results =
            futures::future::try_join_all(cache_entries.into_iter().map(|entry| async move {
                tokio::fs::remove_file(&entry.file_path).await?;

                Ok::<_, std::io::Error>(entry)
            }))
            .await?;

        for entry in cache_results {
            eprintln!("Removed: {}", entry.url);
        }

        eprintln_bright_yellow!("Deleted {total_entires} files, {total_size}");

        Ok(())
    }
}
