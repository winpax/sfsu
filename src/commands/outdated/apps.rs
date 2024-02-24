use clap::Parser;
use rayon::prelude::*;
use sfsu::{
    buckets::Bucket,
    calm_panic::calm_panic,
    output::structured::Structured,
    packages::{install, outdated},
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    pub(super) json: bool,
}

impl super::super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let apps = install::Manifest::list_all_unchecked()?;

        let mut outdated: Vec<outdated::Info> = apps
            .par_iter()
            .flat_map(|app| -> anyhow::Result<outdated::Info> {
                if let Some(bucket) = &app.bucket {
                    let local_manifest = app.get_manifest()?;
                    // TODO: Add the option to check all buckets and find the highest version (will require semver to order versions)
                    let bucket = Bucket::from_name(bucket)?;

                    Ok(bucket.get_manifest(&app.name).map(|remote_manifest| {
                        if let Some(info) =
                            outdated::Info::from_manifests(&local_manifest, &remote_manifest)
                        {
                            info
                        } else {
                            calm_panic("Error: no update available");
                        }
                    })?)
                } else {
                    anyhow::bail!("no bucket specified")
                }
            })
            .collect();

        if outdated.is_empty() {
            println!("No outdated packages.");
        } else {
            outdated.dedup();
            outdated.par_sort_by(|a, b| a.name.cmp(&b.name));

            let values = outdated
                .par_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            if self.json {
                let output = serde_json::to_string_pretty(&values)?;

                println!("{output}");
            } else {
                // TODO: Add a better way to add colours than this
                // TODO: p.s this doesnt work atm
                // use colored::Colorize;
                // let values = values
                //     .into_par_iter()
                //     .map(|mut value| {
                //         if let Some(current) = value.get_mut("Current") {
                //             *current = current.as_str().unwrap().red().to_string().into();
                //         }

                //         if let Some(available) = value.get_mut("Available") {
                //             *available = available.as_str().unwrap().green().to_string().into();
                //         }

                //         value
                //     })
                //     .collect::<Vec<_>>();

                let outputs =
                    Structured::new(&["Name", "Current", "Available"], &values).with_max_length(30);

                print!("{outputs}");
            }
        }

        Ok(())
    }
}
