use std::{
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use serde::Serialize;
use crate::{config, contexts::ScoopContext};
use tokio::task::JoinSet;

mod list;
mod remove;

use crate::{abandon, commands::CommandRunner, matching::PatternMatcher, wrappers::sizes::Size};

use super::Runnable;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
enum CacheEntry {
    Known {
        #[serde(skip)]
        file_path: PathBuf,
        name: String,
        version: String,
        size: Size,
        hash: String,
    },
    #[serde(skip)]
    Loose { file_path: PathBuf, size: Size },
}

impl CacheEntry {
    pub async fn match_paths(
        ctx: &impl ScoopContext,
        patterns: &[String],
        glob: bool,
    ) -> anyhow::Result<Vec<Self>> {
        let cache_path = ctx.cache_path();

        let patterns = patterns
            .iter()
            .filter_map(|pattern| {
                if glob {
                    PatternMatcher::parse_glob(pattern).ok()
                } else {
                    PatternMatcher::parse_regex(pattern).ok()
                }
            })
            .collect::<Vec<_>>();

        let mut set = JoinSet::new();
        let mut dir = tokio::fs::read_dir(cache_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if !patterns.iter().any(|pattern| pattern.test(&file_name)) {
                continue;
            }

            let file_name = file_name.to_string();

            set.spawn(async move {
                fn get_known_info(file_name: &str) -> Option<(String, String, String)> {
                    let mut parts = file_name.split('#');

                    let name = parts.next()?;
                    let version = parts.next()?;
                    let hash = parts.next()?;

                    Some((name.to_string(), version.to_string(), hash.to_string()))
                }

                let metadata = entry.metadata().await?;

                let size = Size::new(metadata.file_size());

                if let Some((name, version, hash)) = get_known_info(&file_name) {
                    debug!("Known cache entry");
                    let cache_entry = CacheEntry::Known {
                        file_path: entry.path(),
                        name,
                        version,
                        hash,
                        size,
                    };

                    anyhow::Ok(cache_entry)
                } else {
                    debug!("Unknown cache entry");
                    anyhow::Ok(CacheEntry::Loose {
                        file_path: entry.path(),
                        size,
                    })
                }
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

    pub fn file_path(&self) -> &Path {
        match self {
            CacheEntry::Known { file_path, .. } | CacheEntry::Loose { file_path, .. } => file_path,
        }
    }

    pub fn size(&self) -> Size {
        match self {
            CacheEntry::Known { size, .. } | CacheEntry::Loose { size, .. } => *size,
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    #[clap(alias = "show", alias = "ls")]
    List(list::Args),
    #[clap(alias = "rm")]
    Remove(remove::Args),
}

impl Runnable for Commands {
    async fn run(
        self,
        ctx: &impl crate::contexts::ScoopContext<Config = crate::config::Scoop>,
    ) -> anyhow::Result<()> {
        match self {
            Commands::List(args) => args.run(ctx).await,
            Commands::Remove(args) => args.run(ctx).await,
        }
    }
}

#[derive(Debug, Clone, Parser)]
/// Show or clear the download cache
pub struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(
        global = true,
        help = "Regex pattern(s) for apps to show cache entries for",
        default_value = ".*?"
    )]
    apps: Vec<String>,

    #[clap(
        global = true,
        long,
        help = "Use glob pattern matching rather than regex"
    )]
    glob: bool,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(
        self,
        ctx: &impl ScoopContext<Config = config::Scoop>,
    ) -> Result<(), anyhow::Error> {
        let command = self.command.unwrap_or(Commands::List(list::Args {
            json: self.json,
            apps: self.apps,
            glob: self.glob,
        }));

        command.run(ctx).await
    }
}
