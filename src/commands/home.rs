use clap::Parser;
use sprinkles::{calm_panic::abandon, packages::reference};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to open the homepage for")]
    package: reference::Package,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let manifest = self
            .package
            .first()
            .ok_or(anyhow::anyhow!("Package not found"))?;

        let Some(homepage) = manifest.homepage else {
            abandon!("No homepage found for package");
        };

        open::that_detached(homepage)?;

        Ok(())
    }
}
