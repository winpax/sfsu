use clap::Parser;
use sprinkles::{
    calm_panic::calm_panic,
    output::sectioned::{Children, Section, Sections},
    packages::reference::{self, Package},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to list dependencies for")]
    package: Package,

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
            self.package.set_bucket(bucket)?;
        }

        let manifests = self.package.list_manifests();

        if manifests.is_empty() {
            calm_panic(format!(
                "Could not find any packages matching: {}",
                self.package
            ));
        };

        if self.json {
            println!("{}", serde_json::to_string(&manifests)?);
            return Ok(());
        }

        let output: Sections<reference::ManifestRef> = manifests
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
