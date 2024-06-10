pub mod purge;
#[cfg(feature = "download")]
pub mod download;
pub mod info;
pub mod home;
pub mod outdated;
pub mod list;
pub mod cat;

use clap::{Parser, Subcommand};

use sfsu_macros::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use super::{Command, CommandRunner};

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    /// Show content of specified manifest
    Cat(cat::Args),
    #[cfg(feature = "download")]
    /// Download the specified app.
    Download(download::Args),
    /// Opens the app homepage
    Home(home::Args),
    /// Display information about a package
    Info(info::Args),
    /// List all installed packages
    List(list::Args),
    /// List outdated buckets and/or packages
    Outdated(outdated::Args),
    /// Purge package's persist folder
    Purge(purge::Args),
}

#[derive(Debug, Clone, Parser)]
/// Commands for managing apps
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

impl Command for Args {
    #[inline]
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        self.command.run(ctx).await
    }
}
