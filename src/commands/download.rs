use anyhow::Context;
use clap::Parser;

use indicatif::MultiProgress;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::blocking::Client;
use sfsu::{
    cache::{Downloader, Handle},
    packages::reference::Package,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The package to download")]
    package: Package,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let manifest = self.package.manifest().context("Failed to find manifest")?;

        let mut m = MultiProgress::new();
        let client = Client::new();

        let downloaders =
            Handle::open_manifest(&manifest, None).context("missing download urls")??;

        let result: std::io::Result<Vec<_>> = downloaders
            .into_par_iter()
            .map(|dl| Downloader::new(dl, &client, &m).unwrap())
            .map(Downloader::download)
            .collect();

        result?;

        Ok(())
    }
}
