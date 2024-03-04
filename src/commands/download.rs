use anyhow::Context;
use clap::Parser;

use indicatif::MultiProgress;
use reqwest::blocking::Client;
use sfsu::{
    cache::{Downloader, Handle},
    packages::reference::{ManifestRef, Package},
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

        let mp = MultiProgress::new();
        let client = Client::new();

        // Note that these are split because it helps the downloads run in parallel

        let downloaders = Handle::open_manifest(&manifest, None)
            .context("missing download urls")??
            .into_iter()
            .map(|dl| Downloader::new(dl, &client, &mp).unwrap())
            .collect::<Vec<_>>();

        let threads = downloaders
            .into_iter()
            .map(|dl| std::thread::spawn(|| dl.download()))
            .collect::<Vec<_>>();

        for thread in threads {
            match thread.join() {
                Ok(res) => res?,
                Err(err) => anyhow::bail!("Thread returned with an error: {err:?}"),
            }
        }

        Ok(())
    }
}
