use clap::{Parser, ValueEnum};
use itertools::Itertools;
use strum::IntoEnumIterator;

#[derive(Debug, Default, ValueEnum, Copy, Clone)]
enum Shell {
    #[default]
    Powershell,
    Bash,
    Zsh,
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<super::CommandsRaw>,

    #[clap(short, long, help = "Print hooks for the given shell")]
    shell: Shell,
}

// TODO: Add function to generate hooks

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        let enabled_hooks: Vec<super::CommandsRaw> = super::CommandsRaw::iter()
            .filter(|variant| !self.disable.contains(variant))
            .collect();

        match self.shell {
            Shell::Powershell => {
                print!("function scoop {{ switch ($args[0]) {{ ");

                // I would love to make this all one condition, but Powershell doesn't seem to support that elegantly
                for command in enabled_hooks {
                    print!("'{command}' {{ return sfsu.exe $args }} ");
                }

                print!("default {{ scoop.ps1 @args }} }} }}");
            }
            Shell::Bash | Shell::Zsh => {
                println!("export SCOOP_EXEC=$(which scoop)");

                println!("scoop () {{\n  case $1 in");

                println!(
                    "      ({}) sfsu.exe $@ ;;",
                    enabled_hooks.iter().format(" | ")
                );

                println!("      (*) $SCOOP_EXEC $@ ;;");
                println!("  esac\n}}");
            }
        }

        Ok(())
    }
}
