use clap::Parser;

use sfsu::{
    buckets::Bucket,
    output::{
        sectioned::{Children, Section, Sections, Text},
        NicerBool,
    },
    packages::manifest::PackageLicense,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageInfo {
    name: String,
    description: Option<String>,
    version: String,
    bucket: String,
    website: Option<String>,
    license: Option<PackageLicense>,
    #[serde(rename = "Updated at")]
    updated_at: String,
    #[serde(rename = "Updated by")]
    updated_by: String,
    installed: NicerBool,
    binaries: Option<String>,
    notes: Option<String>,
}

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
        // TODO: Not sure why this works
        let Some((name, bucket, manifest)) = if let Some(bucket_name) = self.bucket {
            let bucket = Bucket::new(&bucket_name)?;

            Some((
                self.package.clone(),
                bucket_name,
                bucket.get_manifest(&self.package)?,
            ))
        } else {
            let bucket = Bucket::list_all()?.first().expect("first bucket");

            match bucket.get_manifest(&self.package) {
                Ok(manifest) => Some((self.package.clone(), bucket.name().to_string(), manifest)),
                Err(_) => None,
            }
        };

        let installed = {
            let apps = sfsu::list_installed_scoop_apps()?;

            apps.iter()
                .find(|app| {
                    app.with_extension("").file_name() == Some(&std::ffi::OsString::from(&name))
                })
                .is_some()
        };

        // TODO: New output that follows the scoop info format
        let pkg_info = PackageInfo {
            name,
            bucket,
            description: manifest.description,
            version: manifest.version,
            website: manifest.homepage,
            license: manifest.license,
            binaries: manifest.bin.map(|b| b.to_vec().join(",")),
            notes: manifest.notes.map(|notes| notes.to_string()),
            installed: NicerBool::new(installed),
            // updated_at: manifest.updated_at,
            // updated_by: manifest.updated_by,
        };
        let title = format!("{name} in \"{bucket}\":");
        // let obj = value.as_object()?;
        // let keys = obj.keys().collect_vec();
        // let values = obj.values().collect_vec();
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

        // println!("No package found with the name \"{}\"", self.package);
        // std::process::exit(1);
    }
}
