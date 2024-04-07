use clap::Parser;
use colored::Colorize;
use itertools::Itertools;

use sprinkles::{
    calm_panic::calm_panic,
    output::{
        structured::vertical::VTable,
        wrappers::{
            alias_vec::AliasVec,
            bool::{wrap_bool, NicerBool},
            time::NicerTime,
        },
    },
    packages::{
        info::PackageInfo,
        manifest::{StringOrArrayOfStrings, StringOrArrayOfStringsOrAnArrayOfArrayOfStrings},
        reference, Manifest,
    },
    semver, Scoop,
};

#[derive(Debug, Clone, Parser)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    #[clap(help = "The package to get info from")]
    package: reference::Package,

    #[cfg(not(feature = "v2"))]
    #[clap(
        short,
        long,
        help = format!("The bucket to exclusively search in. {}", "DEPRECATED: Use <bucket>/<package> syntax instead".yellow())
    )]
    bucket: Option<String>,

    #[clap(short = 's', long, help = "Show only the most recent package found")]
    single: bool,

    #[clap(short = 'E', long, help = "Show `Updated by` user emails")]
    hide_emails: bool,

    #[clap(from_global)]
    json: bool,

    #[clap(from_global)]
    disable_git: bool,

    #[clap(long, help = "Disable updated info")]
    disable_updated: bool,
}

impl super::Command for Args {
    fn runner(mut self) -> Result<(), anyhow::Error> {
        #[cfg(not(feature = "v2"))]
        if self.package.bucket().is_none() {
            if let Some(bucket) = &self.bucket {
                self.package.set_bucket(bucket.clone());
            }
        }

        let manifests = self.package.list_manifests();

        if manifests.is_empty() {
            calm_panic(format!(
                "No package found with the name \"{}\"",
                self.package
            ));
        }

        if manifests.len() > 1 {
            println!(
                "Found {} packages, matching \"{}\":",
                manifests.len(),
                self.package
            );
        }

        let installed_apps = Scoop::installed_apps()?;

        if self.single {
            let latest = manifests
                .into_iter()
                // .flat_map(|m| semver::Version::parse(&m.version))
                .max_by(|a_manifest, b_manifest| {
                    semver::Version::parse(&a_manifest.version)
                        .and_then(|a_version| {
                            Ok(a_version.cmp(&semver::Version::parse(&b_manifest.version)?))
                        })
                        .unwrap_or(std::cmp::Ordering::Equal)
                }).expect("something went terribly wrong (no manifests found even though we just checked for manifests)");

            self.print_manifest(latest, &installed_apps)?;
        } else {
            for manifest in manifests {
                self.print_manifest(manifest, &installed_apps)?;
            }
        }

        Ok(())
    }
}

impl Args {
    fn print_manifest(
        &self,
        manifest: Manifest,
        installed_apps: &[std::path::PathBuf],
    ) -> anyhow::Result<()> {
        let install_path = {
            let install_path = installed_apps.iter().find(|app| {
                app.with_extension("").file_name()
                    == Some(&std::ffi::OsString::from(&manifest.name))
            });

            install_path.cloned()
        };

        let (updated_at, updated_by) = if self.disable_updated {
            (None, None)
        } else {
            match manifest.last_updated_info(self.hide_emails, self.disable_git) {
                Ok(v) => v,
                Err(_) => match install_path {
                    Some(ref install_path) => {
                        let updated_at = install_path.metadata()?.modified()?;

                        (Some(NicerTime::from(updated_at).to_string()), None)
                    }
                    _ => (None, None),
                },
            }
        };

        let pkg_info = PackageInfo {
            name: manifest.name,
            bucket: manifest.bucket,
            description: manifest.description,
            version: manifest.version.to_string(),
            website: manifest.homepage,
            license: manifest.license,
            binaries: manifest.bin.map(|b| match b {
                StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::String(bin) => bin.to_string(),
                StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::StringArray(bins) => {
                    bins.join(" | ")
                }
                StringOrArrayOfStringsOrAnArrayOfArrayOfStrings::UnionArray(bins) => bins
                    .into_iter()
                    .map(|bin_union| match bin_union {
                        StringOrArrayOfStrings::String(bin) => bin,
                        StringOrArrayOfStrings::StringArray(mut bin_alias) => bin_alias.remove(0),
                    })
                    .join(" | "),
            }),
            notes: manifest
                .notes
                .map(|notes| notes.to_string())
                .unwrap_or_default(),
            installed: wrap_bool!(install_path.is_some()),
            shortcuts: manifest.install_config.shortcuts.map(AliasVec::from_vec),
            updated_at,
            updated_by,
        };

        let value = serde_json::to_value(pkg_info)?;
        if self.json {
            let output = serde_json::to_string_pretty(&value)?;
            println!("{output}");
        } else {
            let table = VTable::new(&value);
            println!("{table}");
        }

        Ok(())
    }
}
