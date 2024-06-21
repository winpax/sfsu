use clap::Parser;
use sprinkles::{contexts::ScoopContext, packages::reference::package};

use crate::abandon;

#[derive(Debug, Clone, Parser)]
/// Opens the app homepage
pub struct Args {
    #[clap(help = "The package to open the homepage for")]
    package: package::Reference,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext) -> Result<(), anyhow::Error> {
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
