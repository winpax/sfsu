use clap::Parser;

#[derive(Debug, Parser)]
struct ListArgs {
    #[clap(long, help = "Disable the `scoop search` hook")]
    no_search: bool,

    #[clap(long, help = "Disable the `scoop list` hook")]
    no_list: bool,
}

fn main() {
    let args = ListArgs::parse();

    print!("function scoop {{ ");

    if !args.no_search {
        print!("if ($args[0] -eq 'search') {{ sfss.exe @($args | Select-Object -Skip 1) }} else")
    }

    if !args.no_list {
        print!("if ($args[0] -eq 'list') {{ sfsl.exe @($args | Select-Object -Skip 1) }} else");
    }

    print!(" {{ scoop.ps1 @args }} }}");
}
