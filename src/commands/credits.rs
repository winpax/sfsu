use clap::Parser;

mod contributors {
    include!(concat!(env!("OUT_DIR"), "/contributors.rs"));
}

mod packages {
    include!(concat!(env!("OUT_DIR"), "/packages.rs"));
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self) -> anyhow::Result<()> {
        todo!()
    }
}
