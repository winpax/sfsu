use clap::Parser;
use sprinkles::{config, contexts::ScoopContext, packages::reference};

use crate::abandon;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to open the homepage for")]
    package: reference::package::Reference,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        let manifest = self
            .package
            .first(ctx)
            .ok_or(anyhow::anyhow!("Package not found"))?;

        let Some(homepage) = manifest.homepage else {
            abandon!("No homepage found for package");
        };

        open::that_detached(homepage)?;

        Ok(())
    }
}
