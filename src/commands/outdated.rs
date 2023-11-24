use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;
use sfsu::{output::structured::Structured, packages::Manifest};

#[derive(Debug, Clone, Parser)]
/// List outdated packages
pub struct Args;

impl super::Command for Args {
    fn run(self) -> anyhow::Result<()> {
        let apps = Manifest::list_installed()?;

        let buckets = sfsu::buckets::Bucket::list_all()?;

        let outdated: Vec<Outdated> = apps
            .par_iter()
            .flat_map(|app| {
                buckets
                    .par_iter()
                    .filter_map(|bucket| match bucket.get_manifest(&app.name) {
                        Ok(manifest) if manifest.version != app.version => Some(Outdated {
                            name: app.name.clone(),
                            current: app.version.clone(),
                            available: manifest.version.clone(),
                        }),
                        _ => None,
                    })
            })
            .collect();

        if outdated.is_empty() {
            println!("No outdated packages.");
        } else {
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Outdated {
    name: String,
    current: String,
    available: String,
}
