use clap::Parser;
use colored::Colorize as _;
use sfsu::{buckets, packages::PackageReference};

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
        let manifest = if let Some(manifest) = self.package.manifest() {
            manifest
        } else {
            let Some(manifest) = buckets::Bucket::list_all()?.into_iter().find_map(|bucket| {
                match bucket.get_manifest(self.package.name()) {
                    Ok(manifest) => Some(manifest),
                    Err(_) => None,
                }
            }) else {
                eprintln!("Could not find package: {}", self.package.to_string().red());
                std::process::exit(1);
            };

            manifest
        };

        dbg!(manifest.depends());

        Ok(())
    }
}

// note to self, use `phpstudy-lagecy-scoop` to test this command
