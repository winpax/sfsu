use clap::Parser;
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(from_global)]
    global: bool,
}

impl super::Command for Args {
    fn needs_elevation(&self) -> bool {
        self.global
    }

    async fn runner(self, _ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        todo!()
    }
}
