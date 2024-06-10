use clap::Parser;

use sprinkles::{buckets::Bucket, config, contexts::ScoopContext, packages::Manifest};

use crate::{
    commands::{DeprecationMessage, DeprecationWarning},
    output::sectioned::{Children, Section, Sections, Text},
};

#[derive(Debug, Clone, Parser)]
/// Describe a package
pub struct Args {
    #[clap(help = "The package to describe")]
    package: String,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,
}

impl super::Command for Args {
    const DEPRECATED: Option<DeprecationWarning> = Some(DeprecationWarning {
        message: DeprecationMessage::Replacement("sfsu info"),
        version: Some(2.0),
    });

    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> Result<(), anyhow::Error> {
        let buckets = Bucket::one_or_all(ctx, self.bucket)?;

        let manifests: Vec<(String, String, Manifest)> = buckets
            .iter()
            .filter_map(|bucket| match bucket.get_manifest(&self.package) {
                Ok(manifest) => Some((self.package.clone(), bucket.name().to_string(), manifest)),
                Err(_) => None,
            })
            .collect();

        let sectioned = manifests
            .iter()
            .map(|(package, bucket, manifest)| {
                let title = format!("{package} in \"{bucket}\":");

                let mut description: Vec<Text<String>> = vec![];

                if let Some(ref pkg_description) = manifest.description {
                    description.push(pkg_description.clone().into());
                }

                description.push(format!("Version: {}", manifest.version).into());

                if let Some(ref homepage) = manifest.homepage {
                    description.push(format!("Homepage: {homepage}").into());
                }
                if let Some(ref license) = manifest.license {
                    description.push(format!("License: {license}").into());
                }

                Section::new(Children::Multiple(description)).with_title(title)
            })
            .collect::<Sections<_>>();

        print!("{sectioned}");

        Ok(())
    }
}
