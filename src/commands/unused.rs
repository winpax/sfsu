use std::fs::read_dir;

use clap::Parser;

use sfsu::{
    output::sectioned::{Children, Section},
    packages::{CreateManifest, InstallManifest},
    Scoop,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    // TODO: Add json option
    // #[clap(from_global)]
    // json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let buckets_path = Scoop::buckets_path();
        let apps_path = Scoop::apps_path();

        let apps = read_dir(apps_path)?.collect::<Result<Vec<_>, _>>()?;

        // TODO: Refactor
        let used_buckets = apps
            .iter()
            .filter_map(|entry| {
                let install_path = entry.path().join("current/install.json");

                if let Ok(InstallManifest {
                    bucket: Some(bucket),
                    ..
                }) = InstallManifest::from_path(install_path)
                {
                    Some(bucket)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let unused_buckets = read_dir(buckets_path)?
            .filter_map(|dir| {
                if let Ok(dir) = dir {
                    let dir_name = dir.file_name();
                    let dir_name_str = dir_name.to_string_lossy().to_string();

                    if !dir.path().is_dir() || used_buckets.contains(&dir_name_str) {
                        None
                    } else {
                        Some(dir_name_str + "\n")
                    }
                } else {
                    None
                }
            })
            .collect::<Children<_>>();

        if let Children::None = unused_buckets {
            println!("No unused buckets");
        } else {
            let unused = Section::new(unused_buckets)
                .with_title("The following buckets are unused:".to_string());
            println!("{unused}");
        };

        Ok(())
    }
}
