use clap::Parser;

use sfsu::{
    buckets::Bucket,
    output::sectioned::{Children, Section, Sections, Text},
    packages::Manifest,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to describe")]
    package: String,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,
}

impl super::Command for Args {
    fn deprecated() -> Option<&'static str> {
        Some("Use `sfsu info` instead. Will be removed in v2")
    }

    fn runner(self) -> Result<(), anyhow::Error> {
        let buckets = Bucket::one_or_all(self.bucket)?;

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
