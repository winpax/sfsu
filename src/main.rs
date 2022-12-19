use clap::{Parser, Subcommand};

use sfsu::commands::*;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Search(search::Args),
    List(list::Args),
    Hook(hook::Args),
}

fn main() {
    _ = Args::parse();
}
