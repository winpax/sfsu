use clap::Parser;
use itertools::Itertools as _;
use serde::Serialize;
use sfsu::{
    buckets::Bucket,
    output::{
        structured::vertical::VTable,
        wrappers::{
            bool::{wrap_bool, NicerBool},
            time::NicerTime,
        },
    },
    packages::{manifest::PackageLicense, Manifest},
    Scoop,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct PackageInfo {
    name: String,
    description: Option<String>,
    version: String,
    bucket: String,
    website: Option<String>,
    license: Option<PackageLicense>,
    #[serde(rename = "Updated at")]
    updated_at: Option<NicerTime>,
    // #[serde(rename = "Updated by")]
    // updated_by: Option<String>,
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
        let buckets = Bucket::one_or_all(self.bucket)?;

        let manifests: Vec<(String, String, Manifest)> = buckets
            .iter()
            .filter_map(|bucket| match bucket.get_manifest(&self.package) {
                Ok(manifest) => Some((self.package.clone(), bucket.name().to_string(), manifest)),
                Err(_) => None,
            })
            .collect();

        if manifests.is_empty() {
            println!("No package found with the name \"{}\"", self.package);
            std::process::exit(1);
        }

        // TODO: Fix execution time

        if manifests.len() > 1 {
            println!(
                "Found {} packages, matching \"{}\":",
                manifests.len(),
                self.package
            );
        }

        let installed_apps = Scoop::list_installed_scoop_apps()?;

        for (name, bucket, manifest) in manifests {
            let install_path = {
                let install_path = installed_apps.iter().find(|app| {
                    app.with_extension("").file_name() == Some(&std::ffi::OsString::from(&name))
                });

                install_path.cloned()
            };

            let updated_at = match install_path {
                Some(ref install_path) => {
                    let updated_at = install_path.metadata()?.modified()?;

                    // TODO: Implement updated_by?
                    Some(updated_at.into())
                }
                _ => None,
            };

            let pkg_info = PackageInfo {
                name,
                bucket,
                description: manifest.description,
                version: manifest.version,
                website: manifest.homepage,
                license: manifest.license,
                binaries: manifest.bin.map(|b| b.to_vec().join(",")),
                notes: manifest.notes.map(|notes| notes.to_string()),
                installed: wrap_bool!(install_path.is_some()),
                updated_at,
            };

            // TODO: Add custom derive macro that allows this without serde_json
            let value = serde_json::to_value(pkg_info).unwrap();
            let obj = value.as_object().expect("valid object");

            let keys = obj.keys().cloned().collect_vec();
            let values = obj
                .values()
                .cloned()
                .map(|v| v.to_string().trim_matches('"').to_string())
                .collect_vec();

            let table = VTable::new(&keys, &values);
            println!("{table}");
        }

        Ok(())
    }
}
