use crate::{contexts::ScoopContext, shell::Shell};
use clap::Parser;
use quork::traits::list::ListVariants;
use std::process::Command as SysCommand;

use super::CommandHooks as CommandsHooks;

#[derive(Debug, Clone, Parser)]
/// Generate hooks for the given shell
pub struct Args {
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<CommandsHooks>,

    #[clap(short = 'E', long, help = "The commands to exclusively enable")]
    enabled: Vec<CommandsHooks>,

    #[clap(short, long, help = "Print hooks for the given shell", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl super::Command for Args {
    async fn runner(self, _: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let shell = self.shell;
        let shell_config = shell.config();
        let enabled_hooks: Vec<CommandsHooks> = {
            // Explicit binding here fixes type inference, as we explicitly cast it to a slice
            let enabled_hooks: &[CommandsHooks] = if self.enabled.is_empty() {
                &CommandsHooks::VARIANTS
            } else {
                &self.enabled
            };

            enabled_hooks
        }
        .iter()
        .filter(|variant| !self.disable.contains(variant))
        .copied()
        .collect();

        match shell {
            Shell::Powershell => {
                print!("function scoop {{ switch ($args[0]) {{ ");

                // I would love to make this all one condition, but Powershell doesn't seem to support that elegantly
                for command in enabled_hooks {
                    print!(
                        "  '{hook}' {{ return sfsu.exe {command} @($args | Select-Object -Skip 1) }} ",
                        hook = command.hook(),
                        command = command.command()
                    );
                }

                println!("default {{ scoop.ps1 @args }} }} }}");

                println!("# To add this to your config, add the following line to the end of your PowerShell profile:");
                println!("#     Invoke-Expression (&sfsu hook)");
                println!("# You can also optionally disable certain hooks via the --disable <COMMAND> flag");
                println!("#     Invoke-Expression (&sfsu hook --disable list)");

                // Detect WSL and print Bash/Zsh snippet based on defaults and opt-out env vars
                let has_wsl = which::which("wsl").is_ok() || which::which("wsl.exe").is_ok();
                let auto_hook_disabled = std::env::var("SFSU_DISABLE_AUTO_HOOK")
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(false);
                let wsl_disabled = std::env::var("SFSU_DISABLE_WSL_AUTO_HOOK")
                    .map(|v| v == "1" || v.to_lowercase() == "true")
                    .unwrap_or(false);

                if has_wsl && !auto_hook_disabled && !wsl_disabled {
                    println!("# WSL detected: installer will add the following to your WSL ~/.bashrc by default (set SFSU_DISABLE_WSL_AUTO_HOOK=1 to opt-out):");
                    println!("#   source <(sfsu.exe hook --shell bash)");
                } else if has_wsl {
                    println!("# WSL detected: automatic WSL hook is disabled. To enable, unset SFSU_DISABLE_WSL_AUTO_HOOK or set it to 0.");
                    println!("#   source <(sfsu.exe hook --shell bash)");
                }

                // Detect Nushell on host and in WSL (if present) and print instructions when found
                let nu_in_host = which::which("nu").is_ok();
                let mut nu_in_wsl = false;
                if has_wsl {
                    nu_in_wsl = SysCommand::new("wsl")
                        .args(&["which", "nu"]) 
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                }

                if nu_in_host || nu_in_wsl {
                    println!("# Nushell is also supported. Run the following command save it to a file.");
                    println!("#   sfsu hook --shell nu | save -f path/to/some/file.nu");
                    println!("# Then source it in your config.nu (situated in path $nu.config-path). ");
                    println!("#   source path/to/the/file.nu");
                }
            }
            Shell::Bash | Shell::Zsh => {
                println!(
                    "SCOOP_EXEC=$(which scoop) \n\
                    scoop () {{ \n\
                    case $1 in"
                );

                for command in enabled_hooks {
                    println!(
                        "({hook}) sfsu.exe {command} ${{@:2}} ;;",
                        hook = command.hook(),
                        command = command.command()
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
                        "def --wrapped \"scoop {hook}\" [...rest] {{ sfsu {command} ...$rest }}",
                        hook = command.hook(),
                        command = command.command()
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
