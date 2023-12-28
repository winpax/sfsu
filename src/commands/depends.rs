use clap::Parser;
use colored::Colorize as _;
use sfsu::{
    output::sectioned::{Children, Section, Sections},
    packages::PackageReference,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to list dependencies for")]
    package: PackageReference,

    #[clap(help = "The bucket of the given package")]
    bucket: Option<String>,

    // TODO: Implement recursion?
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
        let manifests = self.package.search_manifest();

        if manifests.is_empty() {
            eprintln!(
                "Could not find any packages matching: {}",
                self.package.to_string().red()
            );
            std::process::exit(1);
        };

        if self.json {
            println!("{}", serde_json::to_string(&manifests)?);
            return Ok(());
        }

        let output: Sections<String> = manifests
            .into_iter()
            .map(|manifest| {
                let children = Children::from(manifest.depends());
                match children {
                    Children::None => Section::new(children).with_title(format!(
                        "No dependencies found for {} in {}",
                        manifest.name.to_string().red(),
                        manifest.bucket.to_string().red()
                    )),
                    _ => Section::new(children).with_title(format!(
                        "Dependencies for {} in {}",
                        manifest.name.to_string().green(),
                        manifest.bucket.to_string().green()
                    )),
                }
            })
            .collect();

        println!("{output}");

        Ok(())
    }
}

// note to self, use `phpstudy-lagecy-scoop` to test this command
