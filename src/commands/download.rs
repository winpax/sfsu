use std::thread;

use clap::Parser;
use indicatif::MultiProgress;

use itertools::Itertools;
use sprinkles::{
    abandon,
    cache::{Downloader, Handle},
    hash::encode_hex,
    packages::reference::Package,
    requests::BlockingClient,
    Scoop,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The packages to download")]
    packages: Vec<Package>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    const BETA: bool = true;

    async fn runner(self) -> Result<(), anyhow::Error> {
        let mp = MultiProgress::new();
        let client = BlockingClient::new();

        let downloaders = self
            .packages
            .into_iter()
            .map(|package| {
                eprintln!("Downloading {}...", package.name().unwrap_or_default());

                if package.version.is_some() {
                    eprint!("Attempting to generate manifest");
                }
                let manifest = package.manifest()?;
                if let Some(version) = &package.version {
                    eprint!("\r📜 Generated manifest for version {version}");
                    eprintln!();
                }

                let dl = Handle::open_manifest(Scoop::cache_path(), &manifest)?;
                let output_file = dl.file_name.clone();

                match Downloader::new(dl, &client, Some(&mp)) {
                    Ok(dl) => anyhow::Ok(dl),
                    Err(e) => match e {
                        sprinkles::cache::Error::ErrorCode(status) => {
                            abandon!("Found {status} error while downloading")
                        }
                        _ => Err(e.into()),
                    },
                }
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        // let threads = downloaders
        //     .into_iter()
        //     .map(|dl| thread::spawn(move || dl.download()))
        //     .collect_vec();

        // if let Some(actual_hash) = manifest.install_config().hash {
        //     let hash = encode_hex(&hash);
        //     if actual_hash == hash {
        //         eprintln!("🔒 Hash matched: {hash}");
        //     } else {
        //         abandon!("🔓 Hash mismatch: expected {actual_hash}, found {hash}");
        //     }
        // } else {
        //     warn!("🔓 No hash provided, skipping hash check");
        // }

        // eprintln!(
        //     "✅ Downloaded {} to {}",
        //     manifest.name,
        //     output_file.display()
        // );

        Ok(())
    }
}
