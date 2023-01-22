use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,
}

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        todo!()
    }
}
