use rayon::prelude::*;

use clap::Parser;
use colored::Colorize;

use sfsu::{output::structured::Structured, summary::package};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(
        help = format!("The pattern to search for (can be a regex). {}", "DEPRECATED: Use sfsu search --installed. Will be removed in v2".yellow())
    )]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively list packages in")]
    bucket: Option<String>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        let outputs = package::Summary::parse_all()?
            .into_par_iter()
            .filter(|package| {
                if let Some(ref bucket) = self.bucket {
                    return &package.source == bucket;
                }
                // Keep errors so that the following line will return the error
                true
            })
            .collect::<Vec<_>>();

        if self.json {
            let output_json = serde_json::to_string_pretty(&outputs)?;

            println!("{output_json}");
        } else {
            if outputs.is_empty() {
                println!("No packages found.");
                return Ok(());
            }

            let values = outputs
                .into_iter()
                .map(serde_json::to_value)
                .collect::<Result<Vec<_>, _>>()?;

            let outputs =
                Structured::new(&["name", "version", "source", "updated", "notes"], &values)
                    .with_max_length(30);

            print!("{outputs}");
        }

        Ok(())
    }
}
