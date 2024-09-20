pub mod app;
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
pub mod uninstall;
pub mod update;
pub mod virustotal;

use clap::Subcommand;

use sfsu_macros::{Hooks, Runnable};
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
    const DEPRECATED: Option<DeprecationWarning> = None;

    fn needs_elevation(&self) -> bool {
        false
    }

    async fn runner(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()>;
}

pub trait CommandRunner: Command {
    async fn run(self, ctx: &impl ScoopContext<Config = config::Scoop>) -> anyhow::Result<()>
    where
        Self: Sized,
    {
        if let Some(deprecation_warning) = Self::DEPRECATED {
            eprintln_yellow!("{deprecation_warning}\n");
        }

        if self.needs_elevation() && !quork::root::is_root()? {
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
    Search(search::Args),
    #[cfg(not(feature = "v2"))]
    List(list::Args),
    #[no_hook]
    Hook(hook::Args),
    #[cfg(not(feature = "v2"))]
    UnusedBuckets(bucket::unused::Args),
    Bucket(bucket::Args),
    #[cfg(not(feature = "v2"))]
    Describe(describe::Args),
    #[cfg(not(feature = "v2"))]
    Info(info::Args),
    #[cfg(not(feature = "v2"))]
    Outdated(outdated::Args),
    Depends(depends::Args),
    #[cfg(all(feature = "download", not(feature = "v2")))]
    Download(download::Args),
    Status(status::Args),
    #[cfg_attr(not(feature = "v2"), no_hook)]
    Update(update::Args),
    #[cfg(not(feature = "v2"))]
    Home(home::Args),
    #[cfg(not(feature = "v2"))]
    Cat(cat::Args),
    Export(export::Args),
    Checkup(checkup::Args),
    #[cfg(feature = "download")]
    Cache(cache::Args),
    #[hook_name = "virustotal"]
    #[clap(alias = "virustotal")]
    Scan(virustotal::Args),
    #[no_hook]
    Credits(credits::Args),
    #[cfg(feature = "contexts")]
    Uninstall(uninstall::Args),
    App(app::Args),
    #[no_hook]
    #[cfg(debug_assertions)]
    Debug(debug::Args),
}
