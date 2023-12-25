use clap::Parser;

use rayon::prelude::*;
use sfsu::{
    buckets::Bucket,
    output::sectioned::{Children, Section},
    packages::InstallManifest,
};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    // TODO: Add json option
    // #[clap(from_global)]
    // json: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        // TODO: Refactor
        let used_buckets = InstallManifest::list_all()?
            .par_iter()
            .filter_map(|entry| entry.bucket.clone())
            .collect::<Vec<_>>();

        let unused_buckets = Bucket::list_all()?
            .par_iter()
            .filter_map(|bucket| {
                if used_buckets.contains(&bucket.name().to_string()) {
                    Some((bucket.name() + "\n").to_string())
                } else {
                    None
                }
            })
            .collect::<Children<_>>();

        if let Children::None = unused_buckets {
            println!("No unused buckets");
        } else {
            let unused = Section::new(unused_buckets)
                .with_title("The following buckets are unused:".to_string());
            println!("{unused}");
        };

        Ok(())
    }
}
