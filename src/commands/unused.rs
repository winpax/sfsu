use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {}

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        let scoop_buckets_path = buckets::Bucket::get_path();

        Ok(())
    }
}
