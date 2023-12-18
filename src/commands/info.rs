use clap::Parser;

use sfsu::{
    buckets::Bucket,
    output::sectioned::{Children, Section, Sections, Text},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to get info from")]
    package: String,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(long, help = "Display more information about the package")]
    verbose: bool,
}

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        let manifests = if let Some(bucket_name) = self.bucket {
            let bucket = Bucket::new(&bucket_name)?;

            vec![(
                self.package.clone(),
                bucket_name,
                bucket.get_manifest(&self.package)?,
            )]
        } else {
            let buckets = Bucket::list_all()?;

            buckets
                .iter()
                .filter_map(|bucket| match bucket.get_manifest(&self.package) {
                    Ok(manifest) => {
                        Some((self.package.clone(), bucket.name().to_string(), manifest))
                    }
                    Err(_) => None,
                })
                .collect()
        };

        if let Some((name, bucket, manifest)) = manifests.first() {
            // TODO: New output that follows the scoop info format
            let title = format!("{name} in \"{bucket}\":");

            let mut description: Vec<Text<String>> = vec![];

            if let Some(ref pkg_description) = manifest.description {
                description.push(format!("{pkg_description}\n").into());
            }

            description.push(format!("Version: {}\n", manifest.version).into());

            if let Some(ref homepage) = manifest.homepage {
                description.push(format!("Homepage: {homepage}\n").into());
            }
            if let Some(ref license) = manifest.license {
                description.push(format!("License: {license}\n").into());
            }

            // TODO: Maybe multiple children?
            let section = Section::new(Children::Multiple(description)).with_title(title);

            print!("{section}");

            Ok(())
        } else {
            println!("No package found with the name \"{}\"", self.package);
            std::process::exit(1);
        }
    }
}
