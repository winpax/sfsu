use std::{rc::Rc, sync::Arc};

use anyhow::Context;
use clap::Parser;
use rayon::prelude::*;
use sfsu::packages::{CreateManifest, InstallManifest, Manifest};

use crate::ResultIntoOption;

#[derive(Debug, Clone, Parser)]
/// List outdated packages
pub struct Args;

impl super::Command for Args {
    fn run(self) -> anyhow::Result<()> {
        let apps = Manifest::list_installed()?;

        let buckets = sfsu::buckets::Bucket::list_all()?;

        let outdated: Vec<OutdatedPackage> = apps
            .par_iter()
            .flat_map(|app| {
                buckets
                    .par_iter()
                    .filter_map(|bucket| match bucket.get_manifest(&app.name) {
                        Ok(manifest) if manifest.version != app.version => Some(OutdatedPackage {
                            name: app.name.clone(),
                            current: app.version.clone(),
                            available: manifest.version.clone(),
                        }),
                        _ => None,
                    })
            })
            .collect();

        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct OutdatedPackage {
    name: String,
    current: String,
    available: String,
}
