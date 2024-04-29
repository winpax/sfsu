use clap::Parser;
use indicatif::MultiProgress;

use regex::Regex;
use sprinkles::{
    abandon,
    cache::{Downloader, Handle},
    hash::encode_hex,
    packages::reference::Package,
    requests::AsyncClient,
    Architecture, Scoop,
};

#[derive(Debug, Clone, Parser)]
// TODO: Pass architecture
pub struct Args {
    #[clap(help = "The packages to download")]
    packages: Vec<Package>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    const BETA: bool = true;

    async fn runner(self) -> Result<(), anyhow::Error> {
        if self.packages.is_empty() {
            abandon!("No packages provided")
        }

        let mp = MultiProgress::new();

        eprint!("Attempting to generate manifest(s)");
        let downloaders = futures::future::try_join_all(self.packages.into_iter().map(|package| {
            let mp = mp.clone();
            async move {
                let manifest = match package.manifest().await {
                    Ok(manifest) => manifest,
                    Err(e) => abandon!("\rFailed to generate manifest: {e}"),
                };

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
            .map(|(dl, manifest)| tokio::spawn(async move { (dl.download().await, manifest) }));

        let results = futures::future::try_join_all(threads).await?;

        for result in results {
            let (result, manifest) = result;
            let (output_file, hash) = result?;

            eprint!("🔓 Checking {} hash...", manifest.name);

            if let Some(actual_hash) = manifest.install_config(Architecture::ARCH).hash {
                let hash = encode_hex(&hash);
                let actual_hash = Regex::new("(sha5?12?|md5):")
                    .unwrap()
                    .replace(&actual_hash, "");

                if actual_hash == hash {
                    eprintln!("\r🔒 Hash matched: {hash}");
                } else {
                    eprintln!();
                    abandon!("🔓 Hash mismatch: expected {actual_hash}, found {hash}");
                }
            } else {
                eprintln!();
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
