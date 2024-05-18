use clap::Parser;
use sprinkles::{
    config,
    contexts::ScoopContext,
    packages::reference::{self, package},
};

use crate::{
    abandon,
    output::sectioned::{Children, Section, Sections},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to list dependencies for")]
    package: package::Reference,

    #[clap(help = "The bucket of the given package")]
    bucket: Option<String>,

    // TODO: Implement recursion?
    // recursive: bool,
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(mut self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        if let Some(bucket) = self.bucket {
            self.package.set_bucket(bucket)?;
        }

        let manifests = self.package.list_manifests(ctx).await?;

        if manifests.is_empty() {
            abandon!("Could not find any packages matching: {}", self.package);
        };

        if self.json {
            println!("{}", serde_json::to_string(&manifests)?);
            return Ok(());
        }

        let output: Sections<reference::manifest::Reference> = manifests
            .into_iter()
            .filter_map(|manifest| {
                Children::from(manifest.depends())
                    .into_option()
                    .map(|children| {
                        Section::new(children).with_title(format!(
                            "Dependencies for '{}' in '{}'",
                            manifest.name, manifest.bucket
                        ))
                    })
            })
            .collect();

        println!("{output}");

        Ok(())
    }
}

// note to self, use `phpstudy-lagecy-scoop` to test this command
