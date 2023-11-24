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
        let apps = sfsu::list_scoop_apps()?;

        let install_manifests = apps
            .par_iter()
            .map(|app_path| {
                let path = app_path.join("current/install.json");
                match InstallManifest::from_path(&path)
                    .context(format!("{} failed", path.display()))
                {
                    Ok(v) => Ok((
                        app_path
                            .components()
                            .last()
                            .unwrap()
                            .as_os_str()
                            .to_string_lossy()
                            .to_string(),
                        v,
                    )),
                    Err(e) => Err(e),
                }
            })
            .filter_map(|result| match result {
                Ok(e) => Some(e),
                Err(_) => None,
            })
            .collect::<Vec<_>>();

        let buckets = sfsu::buckets::Bucket::list_all()?;

        let apps = buckets
            .into_par_iter()
            .map(|bucket| for app in install_manifests {});
        // .collect::<Result<Vec<_>, _>>()?;

        todo!()
    }
}
