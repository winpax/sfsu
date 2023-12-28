use clap::{Parser, ValueEnum};
use itertools::Itertools;
use strum::{Display, IntoEnumIterator};

#[derive(Debug, Default, ValueEnum, Copy, Clone, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Shell {
    #[default]
    Powershell,
    Bash,
    Zsh,
    Nu,
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<super::CommandsHooks>,

    #[clap(short, long, help = "Print hooks for the given shell", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl super::Command for Args {
    fn runner(self) -> Result<(), anyhow::Error> {
        let shell = self.shell;
        let enabled_hooks: Vec<super::CommandsHooks> = super::CommandsHooks::iter()
            .filter(|variant| !self.disable.contains(variant))
            .collect();

        match shell {
            Shell::Powershell => {
                print!("function scoop {{ switch ($args[0]) {{ ");

                // I would love to make this all one condition, but Powershell doesn't seem to support that elegantly
                for command in enabled_hooks {
                    print!("  '{command}' {{ return sfsu.exe $args }} ");
                }

                println!("default {{ scoop.ps1 @args }} }} }}");

                println!("# To add this to your config, add the following line to the end of your PowerShell profile:");
                println!("#     Invoke-Expression (&sfsu hook)");
            }
            Shell::Bash | Shell::Zsh => {
                let hook_list = enabled_hooks.iter().format(" | ");

                println!(
                    "SCOOP_EXEC=$(which scoop) \n\
                    scoop () {{ \n\
                    case $1 in \n\
                    ({hook_list}) sfsu.exe $@ ;; \n\
                    (*) $SCOOP_EXEC $@ ;; \n\
                    esac \n\
                    }} \n\n\
                    # Add the following to the end of your ~/.{} \n\
                    #   source <(sfsu.exe hook --shell {shell})",
                    if shell == Shell::Bash {
                        "bashrc"
                    } else {
                        "zshrc"
                    }
                );
            }
            Shell::Nu => {
                for command in enabled_hooks {
                    println!(
                        "def --wrapped \"scoop {command}\" [...rest] {{ sfsu {command} $rest }}"
                    );
                }

                println!(
                        "\n# To add this to your config, run `sfsu hook --shell {shell} | save ~/.cache/sfsu.nu`\n\
                        # And then in your main config add the following line to the end:\n\
                        #   source ~/.cache/sfsu.nu"
                    );
            }
        }

        Ok(())
    }
}
