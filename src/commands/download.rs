use clap::Parser;
use indicatif::MultiProgress;

use sprinkles::{
    abandon,
    cache::{Downloader, Handle},
    hash::encode_hex,
    packages::reference::Package,
    requests::AsyncClient,
    Architecture, Scoop,
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

        eprint!("Attempting to generate manifest(s)");
        let downloaders = futures::future::try_join_all(self.packages.into_iter().map(|package| {
            let mp = mp.clone();
            async move {
                let manifest = package.manifest().await?;

                let dl = Handle::open_manifest(Scoop::cache_path(), &manifest, Architecture::ARCH)?;

                let downloader = match Downloader::new(dl, &AsyncClient::new(), Some(&mp)).await {
                    Ok(dl) => anyhow::Ok(dl),
                    Err(e) => match e {
                        sprinkles::cache::Error::ErrorCode(status) => {
                            abandon!("Found {status} error while downloading")
                        }
                        _ => Err(e.into()),
                    },
                }?;

                anyhow::Ok((downloader, manifest))
            }
        }))
        .await?;
        eprintln!("\r📜 Generated manifest for any and all mismatched versions");

        let threads = downloaders
            .into_iter()
            .map(|(dl, hash)| tokio::spawn(async move { (dl.download().await, hash) }));

        let results = futures::future::try_join_all(threads).await?;

        for result in results {
            let (result, manifest) = result;
            let (output_file, hash) = result?;

            eprintln!("Checking {} hash...", manifest.name);

            if let Some(actual_hash) = manifest.install_config(Architecture::ARCH).hash {
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
        }

        Ok(())
    }
}
