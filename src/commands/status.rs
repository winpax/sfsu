use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,

    #[clap(from_global)]
    verbose: bool,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
