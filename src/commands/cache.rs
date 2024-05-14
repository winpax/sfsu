use std::{os::windows::fs::MetadataExt, path::PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use regex::Regex;
use serde::Serialize;
use sfsu_derive::Runnable;
use sprinkles::{config, contexts::ScoopContext, wrappers::sizes::Size};
use tokio::task::JoinSet;

mod rm;
mod show;

use crate::{abandon, commands::CommandRunner};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct CacheEntry {
    #[serde(skip)]
    file_path: PathBuf,
    name: String,
    version: String,
    size: Size,
    url: String,
}

impl CacheEntry {
    pub async fn match_paths(
        ctx: &impl ScoopContext<config::Scoop>,
        patterns: &[String],
    ) -> anyhow::Result<Vec<Self>> {
        let cache_path = ctx.cache_path();

        let patterns = patterns
            .iter()
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
                    file_path: entry.path(),
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

        Ok(cache_entries)
    }
}

#[derive(Debug, Clone, Subcommand, Runnable)]
enum Commands {
    /// List cache entries
    Show(show::Args),
    /// Remove cache entries
    Rm(rm::Args),
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(
        global = true,
        help = "Glob pattern(s) for apps to show cache entries for",
        default_value = ".*?"
    )]
    apps: Vec<String>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        let command = self.command.unwrap_or(Commands::Show(show::Args {
            json: self.json,
            apps: self.apps,
        }));

        command.run(ctx).await
    }
}
