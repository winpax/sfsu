use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::Parser;
use itertools::Itertools;
use serde::Serialize;
use sfsu::{
    buckets::Bucket,
    calm_panic::calm_panic,
    git::Repo,
    output::{
        structured::vertical::VTable,
        wrappers::{
            alias_vec::AliasVec,
            author::Author,
            bool::{wrap_bool, NicerBool},
            keys::Key,
            time::NicerTime,
        },
    },
    packages::{manifest::PackageLicense, Manifest},
    KeyValue, Scoop,
};

#[derive(Debug, Clone, Serialize, sfsu_derive::KeyValue)]
#[serde(rename_all = "PascalCase")]
struct PackageInfo {
    name: String,
    description: Option<String>,
    version: String,
    bucket: String,
    website: Option<String>,
    license: Option<PackageLicense>,
    updated_at: Option<String>,
    updated_by: Option<String>,
    installed: NicerBool,
    binaries: Option<String>,
    notes: Option<String>,
    shortcuts: Option<AliasVec<String>>,
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to get info from")]
    package: String,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(short = 'E', long, help = "Show `Updated by` user emails")]
    hide_emails: bool,

    #[clap(long, help = "Display more information about the package")]
    verbose: bool,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let buckets = Bucket::one_or_all(self.bucket)?;

        let manifests: Vec<(String, String, Manifest)> = buckets
            .iter()
            .filter_map(|bucket| match bucket.get_manifest(&self.package) {
                Ok(manifest) => Some((self.package.clone(), bucket.name().to_string(), manifest)),
                Err(_) => None,
            })
            .collect();

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

        for (name, bucket, manifest) in manifests {
            let install_path = {
                let install_path = installed_apps.iter().find(|app| {
                    app.with_extension("").file_name() == Some(&std::ffi::OsString::from(&name))
                });

                install_path.cloned()
            };

            let repo =
                Bucket::from_name(&bucket).and_then(|bucket| Ok(Repo::from_bucket(&bucket)?));

            let (updated_at, updated_by) = if let Ok(repo) = repo {
                let latest_commit = repo.latest_commit()?;
                let time = latest_commit.time();
                let author = latest_commit.author();

                let date_time = {
                    let secs = time.seconds();
                    let offset = time.offset_minutes();

                    let naive_time = NaiveDateTime::from_timestamp_opt(secs, 0)
                        .ok_or(anyhow::anyhow!("Invalid time"))?;

                    let offset = FixedOffset::east_opt(offset * 60)
                        .ok_or(anyhow::anyhow!("Invalid timezone offset"))?;

                    DateTime::<FixedOffset>::from_naive_utc_and_offset(naive_time, offset)
                };

                let author_wrapped =
                    Author::from_signature(author).with_show_emails(!self.hide_emails);

                (
                    Some(date_time.to_string()),
                    Some(author_wrapped.to_string()),
                )
            } else {
                match install_path {
                    Some(ref install_path) => {
                        let updated_at = install_path.metadata()?.modified()?;

                        (Some(NicerTime::from(updated_at).to_string()), None)
                    }
                    _ => (None, None),
                }
            };

            let pkg_info = PackageInfo {
                name,
                bucket,
                description: manifest.description,
                version: manifest.version,
                website: manifest.homepage,
                license: manifest.license,
                binaries: manifest.bin.map(|b| b.into_vec().join(",")),
                notes: manifest.notes.map(|notes| notes.to_string()),
                installed: wrap_bool!(install_path.is_some()),
                shortcuts: manifest.install_config.shortcuts.map(AliasVec::from_vec),
                updated_at,
                updated_by,
            };

            if self.json {
                let output = serde_json::to_string_pretty(&pkg_info)?;

                println!("{output}");
            } else {
                let (keys, values) = pkg_info.into_pairs();

                let keys = keys.into_iter().map(Key::wrap).collect_vec();

                let table = VTable::new(&keys, &values);
                println!("{table}");
            }
        }

        Ok(())
    }
}
