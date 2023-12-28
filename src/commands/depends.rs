use clap::Parser;
use colored::Colorize as _;
use sfsu::packages::PackageReference;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to list dependencies for")]
    package: PackageReference,

    #[clap(help = "The bucket of the given package")]
    bucket: Option<String>,

    // TODO: Implement recursion
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
        let Some(manifest) = self.package.search_manifest() else {
            eprintln!("Could not find package: {}", self.package.to_string().red());
            std::process::exit(1);
        };

        dbg!(manifest.depends());

        Ok(())
    }
}

// note to self, use `phpstudy-lagecy-scoop` to test this command
