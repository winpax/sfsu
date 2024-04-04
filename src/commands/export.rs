use clap::Parser;

use sfsu::packages::export::Export;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short, long, help = "Export the scoop config as well")]
    config: bool,
}

impl super::Command for Args {
    fn runner(self) -> anyhow::Result<()> {
        let export = {
            let mut export = Export::load()?;

            if !self.config {
                export.config = None;
            }

            export
        };

        let output = serde_json::to_string_pretty(&export)?;

        println!("{output}");

        Ok(())
    }
}
