use clap::{Parser, ValueEnum};
use itertools::Itertools;
use strum::{Display, IntoEnumIterator};

#[derive(Debug, Default, ValueEnum, Copy, Clone, Display)]
#[strum(serialize_all = "snake_case")]
enum Shell {
    #[default]
    Powershell,
    Bash,
    Zsh,
    Nu,
}

#[derive(Debug, Clone, Parser)]
/// Generate hooks for the given shell
pub struct Args {
    // TODO: Rename CommandsHooks to CommandHooks or something
    // TODO: Add attribute macro that excludes a variant from the aforementioned enum
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<super::CommandsHooks>,

    #[clap(short, long, help = "Print hooks for the given shell", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        let enabled_hooks: Vec<super::CommandsHooks> = super::CommandsHooks::iter()
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
            Shell::Nu => {
                for command in enabled_hooks {
                    println!(
                        "extern-wrapped \"scoop {command}\" [...rest] {{ sfsu {command} $rest }} "
                    );
                }
            }
        }

        Ok(())
    }
}
