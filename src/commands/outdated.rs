use clap::Parser;

#[derive(Debug, Clone, Parser)]
/// List outdated packages
pub struct Args;

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
