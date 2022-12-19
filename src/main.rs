use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {}

fn main() {
    _ = Args::parse();
}
