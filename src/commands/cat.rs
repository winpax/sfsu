use std::{fs::File, io::Read};

use clap::Parser;
use sprinkles::{calm_panic::abandon, packages::reference};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The manifest to display")]
    package: reference::Package,

    #[clap(from_global)]
    no_color: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let manifests = self.package.list_manifest_paths();

        if manifests.is_empty() {
            abandon!("No manifests found for {}", self.package);
        }

        let manifest = &manifests[0];

        let manifest_content = {
            let mut buf = vec![];

            let mut file = File::open(manifest)?;
            file.read_to_end(&mut buf)?;

            buf
        };

        if self.no_color {
            print!("{}", String::from_utf8_lossy(&manifest_content));
        } else {
            use bat::PrettyPrinter;

            PrettyPrinter::new()
                .input_from_bytes(&manifest_content)
                .language("json")
                .print()?;
        }

        Ok(())
    }
}
