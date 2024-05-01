use clap::Parser;
use itertools::Itertools;

use sprinkles::{
    calm_panic::abandon,
    output::{
        structured::vertical::VTable,
        wrappers::{alias_vec::AliasVec, bool::NicerBool, time::NicerTime},
    },
    packages::{
        info::PackageInfo,
        manifest::{AliasArray, StringArray},
        reference, Manifest, MergeDefaults,
    },
    semver, Architecture, Scoop,
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
        help = format!("The bucket to exclusively search in. {}", console::style("DEPRECATED: Use <bucket>/<package> syntax instead").yellow())
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
    async fn runner(mut self) -> anyhow::Result<()> {
        #[cfg(not(feature = "v2"))]
        if self.package.bucket().is_none() {
            if let Some(bucket) = &self.bucket {
                self.package.set_bucket(bucket.clone())?;
            }
        }

        let manifests = self.package.list_manifests().await?;

        if manifests.is_empty() {
            abandon!("No package found with the name \"{}\"", self.package);
        }

        if manifests.len() > 1 && !self.single {
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
                .max_by(|a_manifest, b_manifest| {
                    semver::Version::try_from(&a_manifest.version)
                        .and_then(|a_version| {
                            Ok(a_version.cmp(&semver::Version::try_from(&b_manifest.version)?))
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
        // TODO: Remove this and just create the pathbuf from the package name
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
            binaries: manifest
                .architecture
                .merge_default(manifest.install_config.clone(), Architecture::ARCH)
                .bin
                .map(|b| match b {
                    AliasArray::NestedArray(StringArray::String(bin)) => bin.to_string(),
                    AliasArray::NestedArray(StringArray::StringArray(bins)) => bins.join(" | "),
                    AliasArray::AliasArray(bins) => bins
                        .into_iter()
                        .map(|bin_union| match bin_union {
                            StringArray::String(bin) => bin,
                            StringArray::StringArray(mut bin_alias) => bin_alias.remove(0),
                        })
                        .join(" | "),
                }),
            notes: manifest
                .notes
                .map(|notes| notes.to_string())
                .unwrap_or_default(),
            installed: NicerBool::new(install_path.is_some()),
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
