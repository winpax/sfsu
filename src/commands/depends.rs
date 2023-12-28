use clap::Parser;
use sfsu::packages::PackageReference;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The name of the package to list dependencies for")]
    package: PackageReference,

    // TODO: Implement recursivity
    // recursive: bool,
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
