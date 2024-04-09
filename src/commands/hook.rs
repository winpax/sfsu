use clap::Parser;
use sprinkles::shell::Shell;
use strum::IntoEnumIterator;

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
        let shell_config = shell.config();
        let enabled_hooks: Vec<super::CommandsHooks> = super::CommandsHooks::iter()
            .filter(|variant| !self.disable.contains(variant))
            .collect();

        match shell {
            Shell::Powershell => {
                print!("function scoop {{ switch ($args[0]) {{ ");

                // I would love to make this all one condition, but Powershell doesn't seem to support that elegantly
                for command in enabled_hooks {
                    print!(
                        "  '{command}' {{ return sfsu.exe {} }} ",
                        command.command_name()
                    );
                }

                println!("default {{ scoop.ps1 @args }} }} }}");

                // TODO: Figure out a way to put these in that PowerShell won't throw a fit about
                // println!("# To add this to your config, add the following line to the end of your PowerShell profile:");
                // println!("#     Invoke-Expression (&sfsu hook)");
            }
            Shell::Bash | Shell::Zsh => {
                println!(
                    "SCOOP_EXEC=$(which scoop) \n\
                    scoop () {{ \n\
                    case $1 in"
                );

                for command in enabled_hooks {
                    println!(
                        "({command}) sfsu.exe {} ${{@:2}} ;;",
                        command.command_name()
                    );
                }

                println!(
                    "(*) $SCOOP_EXEC $@ ;; \n\
                    esac \n\
                    }} \n\n\
                    # Add the following to the end of your ~/.{shell_config} \n\
                    #   source <(sfsu.exe hook --shell {shell})"
                );
            }
            Shell::Nu => {
                for command in enabled_hooks {
                    println!(
                        "def --wrapped \"scoop {command}\" [...rest] {{ sfsu {} ...$rest }}",
                        command.command_name()
                    );
                }

                println!(
                        "\n# To add this to your config, run `sfsu hook --shell {shell} | save ~/.cache/sfsu.nu`\n\
                        # And then in your {shell_config} add the following line to the end:\n\
                        #   source ~/.cache/sfsu.nu"
                    );
            }
        }

        Ok(())
    }
}
