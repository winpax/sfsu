use std::fs::read_dir;

use clap::Parser;

use crate::packages::FromPath;

#[derive(Debug, Parser)]
pub struct Args {}

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        let scoop_buckets_path = crate::buckets::Bucket::get_path();
        let scoop_apps_path = crate::get_scoop_path().join("apps");

        let apps = read_dir(scoop_apps_path)?.collect::<Result<Vec<_>, _>>()?;

        let used_buckets = apps
            .iter()
            .filter_map(|entry| {
                let install_path = entry.path().join("current/install.json");

                if let Ok(install_manifest) =
                    crate::packages::InstallManifest::from_path(&install_path)
                {
                    Some(install_manifest.bucket)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let unused_buckets = read_dir(scoop_buckets_path)?
            .filter_map(|dir| {
                if let Ok(dir) = dir {
                    let dir_name = dir.file_name();
                    let dir_name_str = dir_name.to_string_lossy().to_string();

                    if used_buckets.contains(&dir_name_str) {
                        None
                    } else {
                        Some(dir_name_str)
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        println!("The following buckets are unused: ");
        for bucket in unused_buckets {
            println!("  {bucket}");
        }

        Ok(())
    }
}
