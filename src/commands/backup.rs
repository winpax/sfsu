use clap::Parser;
use sprinkles::packages::reference::Package;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to backup")]
    package: Package,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self) -> anyhow::Result<()> {
        todo!()
    }
}
