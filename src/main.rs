use clap::Parser;

use sfsu::commands::*;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

fn main() {
    let args = Args::parse();
}
