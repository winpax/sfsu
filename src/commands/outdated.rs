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
        let apps = InstallManifest::list_all()?;

        let buckets = sfsu::buckets::Bucket::list_all()?;

        let apps = buckets
            .into_par_iter()
            .map(|bucket| for app in install_manifests {})
            .collect::<Result<Vec<_>, _>>()?;

        todo!()
    }
}
