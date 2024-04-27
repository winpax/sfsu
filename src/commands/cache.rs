use std::os::windows::fs::MetadataExt;

use clap::Parser;
use sprinkles::{output::wrappers::sizes::Size, Scoop};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self) -> Result<(), anyhow::Error> {
        let cache_path = Scoop::cache_path();

        let mut dir = tokio::fs::read_dir(cache_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let metadata = entry.metadata().await?;

            let name = entry.file_name();
            let name = name.to_string_lossy();

            #[allow(clippy::cast_precision_loss)]
            let size = Size::new(metadata.file_size() as f64);
            println!("{name}, {size} bytes");
        }

        Ok(())

        // todo!()
    }
}
