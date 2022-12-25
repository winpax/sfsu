use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long, help = "Disable the `scoop search` hook")]
    no_search: bool,

    #[clap(long, help = "Disable the `scoop list` hook")]
    no_list: bool,

    #[clap(long, help = "Disable the `scoop unused-buckets` hook")]
    no_unused_buckets: bool,
}

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        print!("function scoop {{ ");

        if !self.no_search {
            print!(
                "if ($args[0] -eq 'search') {{ sfsu.exe search @($args | Select-Object -Skip 1) }} else"
            );
        }

        if !self.no_list {
            print!("if ($args[0] -eq 'list') {{ sfsu.exe list --json @($args | Select-Object -Skip 1) | ConvertFrom-Json }} else");
        }

        if !self.no_unused_buckets {
            print!("if ($args[0] -eq 'unused-buckets') {{ sfsu.exe unused-buckets @($args | Select-Object -Skip 1 }} else");
        }

        print!(" {{ scoop.ps1 @args }} }}");

        Ok(())
    }
}
