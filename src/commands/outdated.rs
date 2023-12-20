use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;
use sfsu::{
    buckets::Bucket,
    output::structured::Structured,
    packages::{Manifest, Result as PackageResult},
};

#[derive(Debug, Clone, Parser)]
pub struct Args;

impl super::Command for Args {
    fn run(self) -> anyhow::Result<()> {
        let apps = Manifest::list_installed()?
            .into_iter()
            .filter(std::result::Result::is_ok)
            .collect::<PackageResult<Vec<_>>>()?;

        let mut outdated: Vec<Outdated> = apps
            .par_iter()
            .flat_map(|app| -> anyhow::Result<Outdated> {
                // TODO: Add the option to check all buckets and find the highest version (will require semver to order versions)
                let bucket = Bucket::new(&app.bucket)?;
                match bucket.get_manifest(&app.name) {
                    Ok(manifest) if manifest.version != app.version => Ok(Outdated {
                        name: app.name.clone(),
                        current: app.version.clone(),
                        available: manifest.version.clone(),
                    }),
                    _ => anyhow::bail!("bucket does not have that package"),
                }
            })
            .collect();

        if outdated.is_empty() {
            println!("No outdated packages.");
        } else {
            outdated.dedup();
            let values = outdated
                .par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            let outputs =
                Structured::new(&["Name", "Current", "Available"], &values).with_max_length(30);

            print!("{outputs}");
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Outdated {
    name: String,
    current: String,
    available: String,
}
