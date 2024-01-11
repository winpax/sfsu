use anyhow::Context;
use clap::Parser;

use indicatif::{MultiProgress, ProgressBar};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use reqwest::blocking::Client;
use sfsu::{
    cache::{CacheDownloader, ScoopCache},
    packages::reference::Package,
    SupportedArch,
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
            ScoopCache::open_manifest(&manifest, None).context("missing download urls")??;

        let result: std::io::Result<Vec<_>> = downloaders
            .into_par_iter()
            .map(|dl| CacheDownloader::new(dl, &client, &m).unwrap())
            .map(CacheDownloader::download)
            .collect();

        result?;

        Ok(())
    }
}
