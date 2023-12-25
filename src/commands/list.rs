use clap::Parser;
use colored::Colorize;

use sfsu::{output::structured::Structured, packages::MinInfo};

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
    fn run(self) -> Result<(), anyhow::Error> {
        let outputs = MinInfo::list_installed(self.bucket.as_ref())?;

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
                Structured::new(&["Name", "Version", "Source", "Updated", "Notes"], &values)
                    .with_max_length(30);

            print!("{outputs}");
        }

        Ok(())
    }
}
