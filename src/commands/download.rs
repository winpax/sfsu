use anyhow::Context;
use clap::Parser;

use indicatif::{MultiProgress, ProgressBar};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sfsu::{cache::ScoopCache, packages::reference::Package, SupportedArch};

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

        let downloaders =
            ScoopCache::open_manifest(&manifest, None).context("missing download urls")??;

        // downloaders.into_par_iter().map(|dl| {
        //     let pb = m.add(ProgressBar::new());
        // });

        for dl in downloaders {
            println!("Downloading \"{}\"", dl.file_name.display());
        }

        todo!()
    }
}
