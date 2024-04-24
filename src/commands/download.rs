use anyhow::Context;
use clap::Parser;
use indicatif::MultiProgress;

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
    #[clap(help = "The package to download")]
    package: Package,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let manifest = self.package.manifest().context("Failed to find manifest")?;

        let mp = MultiProgress::new();
        let client = BlockingClient::new();

        let dl = Handle::open_manifest(Scoop::cache_path(), &manifest)
            .context("missing download urls")??;

        let downloader = match Downloader::new(dl, &client, &mp) {
            Ok(dl) => Ok(dl),
            Err(e) => match e {
                sprinkles::cache::Error::ErrorCode(status) => {
                    abandon!("Found {status} error while downloading")
                }
                _ => Err(e),
            },
        }?;

        let (_, hash) = downloader.download()?;

        if let Some(actual_hash) = manifest.install_config().hash {
            let hash = encode_hex(&hash);
            if actual_hash != hash {
                abandon!("Hash mismatch: expected {actual_hash}, got {hash}");
            }
        }

        Ok(())
    }
}
