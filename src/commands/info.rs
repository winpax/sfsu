use std::time::SystemTime;

use clap::Parser;

use itertools::Itertools as _;
use sfsu::{
    buckets::Bucket,
    output::{
        sectioned::{Children, Section, Text},
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
    updated_at: Option<SystemTime>,
    #[serde(rename = "Updated by")]
    updated_by: Option<String>,
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
        let pkg_info = {
            if let Some(bucket_name) = self.bucket {
                let bucket = Bucket::new(&bucket_name)?;

                Some((
                    self.package.clone(),
                    bucket_name,
                    bucket.get_manifest(&self.package)?,
                ))
            } else {
                let buckets = Bucket::list_all()?;
                let bucket = buckets.first().expect("first bucket");

                match bucket.get_manifest(&self.package) {
                    Ok(manifest) => {
                        Some((self.package.clone(), bucket.name().to_string(), manifest))
                    }
                    Err(_) => None,
                }
            }
        };

        let Some((name, bucket, manifest)) = pkg_info.clone() else {
            println!("No package found with the name \"{}\"", self.package);
            std::process::exit(1);
        };

        let install_path = {
            let apps = sfsu::list_installed_scoop_apps()?;

            let install_path = apps.iter().find(|app| {
                app.with_extension("").file_name() == Some(&std::ffi::OsString::from(&name))
            });

            install_path.cloned()
        };

        let (updated_at, updated_by) = if let Some(ref install_path) = install_path {
            let updated_at = install_path.metadata()?.modified()?;
            let updated_by = match crate::file_owner(install_path) {
                Ok(owner) => Some(owner),
                Err(_) => None,
            };

            (Some(updated_at), updated_by)
        } else {
            (None, None)
        };

        // TODO: New output that follows the scoop info format
        let pkg_info = PackageInfo {
            name: name.clone(),
            bucket: bucket.clone(),
            description: manifest.description.clone(),
            version: manifest.version.clone(),
            website: manifest.homepage.clone(),
            license: manifest.license.clone(),
            binaries: manifest.bin.map(|b| b.to_vec().join(",")),
            notes: manifest.notes.map(|notes| notes.to_string()),
            installed: NicerBool::new(install_path.is_some()),
            updated_at,
            updated_by,
        };
        let value = serde_json::to_value(pkg_info)?;
        let obj = value.as_object().expect("valid object");

        let keys = obj.keys().collect_vec();
        let values = obj.values().collect_vec();

        // let title = format!("{name} in \"{bucket}\":");
        // let mut description: Vec<Text<String>> = vec![];

        // if let Some(ref pkg_description) = manifest.description {
        //     description.push(format!("{pkg_description}\n").into());
        // }

        // description.push(format!("Version: {}\n", manifest.version).into());

        // if let Some(ref homepage) = manifest.homepage {
        //     description.push(format!("Homepage: {homepage}\n").into());
        // }
        // if let Some(ref license) = manifest.license {
        //     description.push(format!("License: {license}\n").into());
        // }

        // // TODO: Maybe multiple children?
        // let section = Section::new(Children::Multiple(description)).with_title(title);

        // print!("{section}");

        Ok(())
    }
}
