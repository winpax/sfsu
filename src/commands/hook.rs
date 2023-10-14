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
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<super::CommandsRaw>,

    #[clap(short, long, help = "Print hooks for the given shell", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl super::Command for Args {
    fn run(self) -> Result<(), anyhow::Error> {
        let enabled_hooks: Vec<super::CommandsRaw> = super::CommandsRaw::iter()
            .filter(|variant| !self.disable.contains(variant))
            .collect();

        // TODO: Add helper comments for other shells
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

                println!(
                    "# Add the following to the end of your zshrc \n\
                    #\tsource <(sfsu.exe hook --shell {})",
                    self.shell
                );
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
