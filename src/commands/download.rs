use anyhow::Context;
use clap::Parser;

use indicatif::MultiProgress;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use sfsu::{
    cache::{Downloader, Handle},
    packages::reference::Package,
};
use tokio::runtime::Runtime;

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

        let mp = MultiProgress::new();

        let downloaders =
            Handle::open_manifest(&manifest, None).context("missing download urls")??;

        // TODO: Remove this panic
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        downloaders.into_iter().map(|dl| {
            let mp = mp.clone();
            rt.spawn(async move {
                let dl = dl.begin_download(&Client::new(), &mp).await.unwrap();
                dl.download().await;
            });
        });

        Ok(())
    }
}
