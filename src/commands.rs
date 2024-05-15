pub mod bucket;
pub mod cache;
pub mod cat;
pub mod checkup;
pub mod credits;
pub mod debug;
pub mod depends;
pub mod describe;
#[cfg(feature = "download")]
pub mod download;
pub mod export;
pub mod home;
pub mod hook;
pub mod info;
pub mod list;
#[cfg(not(feature = "v2"))]
pub mod outdated;
pub mod search;
pub mod status;
pub mod update;
pub mod virustotal;

use clap::Subcommand;

use sfsu_derive::{Hooks, Runnable};
use sprinkles::{config, contexts::ScoopContext};

use crate::{abandon, output::colours::eprintln_yellow};

#[derive(Debug, Clone, Copy)]
pub struct DeprecationWarning {
    /// Deprecation message
    message: DeprecationMessage,
    /// Version to be removed in
    version: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeprecationMessage {
    /// Replacement info
    Replacement(&'static str),
    /// Warning message
    Warning(&'static str),
}

impl std::fmt::Display for DeprecationMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeprecationMessage::Replacement(replacement) => {
                write!(f, "Use `{replacement}` instead")
            }
            DeprecationMessage::Warning(warning) => write!(f, "{warning}"),
        }
    }
}

impl std::fmt::Display for DeprecationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DEPRECATED: ")?;

        std::fmt::Display::fmt(&self.message, f)?;

        if let Some(version) = self.version {
            write!(f, "Will be removed in v{version}. ")?;
        }

        Ok(())
    }
}

// TODO: Run command could return `impl Display` and print that itself
pub trait Command {
    const BETA: bool = false;
    const NEEDS_ELEVATION: bool = false;

    const DEPRECATED: Option<DeprecationWarning> = None;

    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()>;
}

pub trait CommandRunner: Command {
    async fn run(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        if let Some(deprecation_warning) = Self::DEPRECATED {
            eprintln_yellow!("{deprecation_warning}\n");
        }

        if Self::NEEDS_ELEVATION && !quork::root::is_root()? {
            abandon!("This command requires elevation. Please run as an administrator.");
        }

        if Self::BETA {
            eprintln_yellow!(
                "This command is in beta and may not work as expected. Please report any and all bugs you find!\n",
            );
        }

        self.runner(ctx).await
    }
}

impl<T: Command> CommandRunner for T {}

#[derive(Debug, Clone, Subcommand, Hooks, Runnable)]
pub enum Commands {
    /// Search for a package
    Search(search::Args),
    /// List all installed packages
    List(list::Args),
    #[no_hook]
    /// Generate hooks for the given shell
    Hook(hook::Args),
    #[cfg(not(feature = "v2"))]
    /// Find buckets that do not have any installed packages
    UnusedBuckets(bucket::unused::Args),
    /// Commands to manage buckets
    Bucket(bucket::Args),
    #[cfg(not(feature = "v2"))]
    /// Describe a package
    Describe(describe::Args),
    /// Display information about a package
    Info(info::Args),
    #[cfg(not(feature = "v2"))]
    /// List outdated buckets and/or packages
    Outdated(outdated::Args),
    /// List the dependencies of a given package, in the order that they will be installed
    Depends(depends::Args),
    #[cfg(feature = "download")]
    /// Download the specified app.
    Download(download::Args),
    /// Show status and check for new app versions
    Status(status::Args),
    #[cfg_attr(not(feature = "v2"), no_hook)]
    /// Update Scoop and Scoop buckets
    Update(update::Args),
    /// Opens the app homepage
    Home(home::Args),
    /// Show content of specified manifest
    Cat(cat::Args),
    /// Exports installed apps, buckets (and optionally configs) in JSON format
    Export(export::Args),
    /// Check for common issues
    Checkup(checkup::Args),
    #[cfg(feature = "download")]
    /// Show or clear the download cache
    Cache(cache::Args),
    /// Scan a file with `VirusTotal`
    Virustotal(virustotal::Args),
    #[no_hook]
    /// Show credits
    Credits(credits::Args),
    #[no_hook]
    #[cfg(debug_assertions)]
    /// Debugging commands
    Debug(debug::Args),
}
