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
    const BETA: bool = true;

    fn runner(self) -> Result<(), anyhow::Error> {
        eprintln!("Downloading {}...", self.package.name().unwrap_or_default());

        if self.package.version.is_some() {
            eprint!("Attempting to generate manifest");
        }
        let manifest = self.package.manifest()?;
        if let Some(version) = &self.package.version {
            eprint!("\r📜 Generated manifest for version {version}");
            eprintln!();
        }

        let mp = MultiProgress::new();
        let client = BlockingClient::new();

        let dl = Handle::open_manifest(Scoop::cache_path(), &manifest)?;
        let output_file = dl.file_name.clone();

        let downloader = match Downloader::new(dl, &client, Some(&mp)) {
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
            if actual_hash == hash {
                eprintln!("🔒 Hash matched: {hash}");
            } else {
                abandon!("🔓 Hash mismatch: expected {actual_hash}, found {hash}");
            }
        } else {
            warn!("🔓 No hash provided, skipping hash check");
        }

        eprintln!(
            "✅ Downloaded {} to {}",
            manifest.name,
            output_file.display()
        );

        Ok(())
    }
}
