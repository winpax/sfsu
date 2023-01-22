#![warn(clippy::all, clippy::pedantic, rust_2018_idioms)]

// TODO: Replace regex with glob
// TODO: Global custom hook fn

mod buckets;
mod commands;
mod config;
mod packages;

use std::path::PathBuf;

use clap::Parser;

use commands::{Command, Commands};

#[must_use]
/// Gets the user's scoop path, via either the default path or as provided by the SCOOP env variable
///
/// Will ignore the global scoop path
///
/// # Panics
/// - There is no home folder
/// - The discovered scoop path does not exist
fn get_scoop_path() -> PathBuf {
    use std::env::var_os;

    // TODO: Add support for both global and non-global scoop installs

    let scoop_path =
        var_os("SCOOP").map_or_else(|| dirs::home_dir().unwrap().join("scoop"), PathBuf::from);

    if scoop_path.exists() {
        dunce::canonicalize(scoop_path).expect("failed to find real path to scoop")
    } else {
        panic!("Scoop path does not exist");
    }
}

#[derive(Debug, Parser)]
#[clap(about, author, version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    args.command.run()
}
