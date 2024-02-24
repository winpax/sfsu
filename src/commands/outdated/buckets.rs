use clap::Parser;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    pub(super) json: bool,
}

impl super::super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
