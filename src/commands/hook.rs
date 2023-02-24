use clap::Parser;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<super::CommandsRaw>,

    #[clap(short, long, help = "Print the WSL/MSYS2 version of the hooks")]
    nix: bool,
}

// TODO: Add function to generate hooks

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        let enabled_hooks: Vec<super::CommandsRaw> = super::CommandsRaw::iter()
            .filter(|variant| !self.disable.contains(variant))
            .collect();

        if self.nix {
            println!("export SCOOP_EXEC=$(which scoop)");

            println!("scoop () {{\ncase $1 in");

            for command in enabled_hooks {
                println!("   ({command}) sfsu.exe {command} $@ ;;");
            }

            println!("  (*) $SCOOP_EXEC $@ ;;");
            println!("  esac\n}}");
        } else {
            print!("function scoop {{ switch ($args[0]) {{ ");

            for command in enabled_hooks {
                print!("'{command}' {{ return sfsu.exe $args }} ");
            }

            print!("default {{ scoop.ps1 @args }} }} }}");
        }

        Ok(())
    }
}
