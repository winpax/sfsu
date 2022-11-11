use clap::Parser;
use sfst::buckets;

#[derive(Debug, Parser)]
pub struct ListArgs {
    #[clap(help = "The pattern to search for (can be a regex)")]
    pattern: Option<String>,

    #[clap(short, long, help = "The bucket to exclusively search in")]
    bucket: Option<String>,

    #[clap(long, help = "Print the Powershell hook")]
    hook: bool,
}

fn main() {
    let scoop_buckets_path = buckets::get_path();
}
