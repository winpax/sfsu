use anyhow::Context;
use clap::Parser;
use sfsu::{buckets, packages::PackageReference};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to list dependencies for")]
    package: PackageReference,

    #[clap(help = "The bucket of the given package")]
    bucket: Option<String>,

    // TODO: Implement recursivity
    // recursive: bool,
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(mut self) -> Result<(), anyhow::Error> {
        if let Some(bucket) = self.bucket {
            self.package.set_bucket(bucket);
        }

        // TODO: Search buckets for the first match, but warn of this
        let manifest = self.package.manifest().context("Failed to get manifest")?;

        dbg!(manifest.depends());

        Ok(())
    }
}

// note to self, use `phpstudy-lagecy-scoop` to test this command