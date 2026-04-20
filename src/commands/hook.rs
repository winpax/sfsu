use crate::{contexts::ScoopContext, shell::Shell};
use clap::{Parser, Subcommand};
use quork::traits::list::ListVariants;
use std::process::Command as SysCommand;

use super::CommandHooks as CommandsHooks;

#[derive(Debug, Clone, Subcommand)]
pub enum HookCommands {
    #[clap(name = "powershell", alias = "pwsh")]
    /// Install the hook for PowerShell automatically
    Powershell {
        #[clap(short, long, help = "Print the hook instead of installing")]
        print: bool,
    },
    #[clap(name = "bash")]
    /// Install the hook for Bash automatically
    Bash {
        #[clap(short, long, help = "Print the hook instead of installing")]
        print: bool,
    },
    #[clap(name = "zsh")]
    /// Install the hook for Zsh automatically
    Zsh {
        #[clap(short, long, help = "Print the hook instead of installing")]
        print: bool,
    },
    #[clap(name = "nu")]
    /// Install the hook for Nushell automatically
    Nu {
        #[clap(short, long, help = "Print the hook instead of installing")]
        print: bool,
    },
    #[clap(name = "wsl")]
    /// Install the hook for WSL automatically
    Wsl {
        /// The WSL distro to install in (defaults to the default distro)
        distro: Option<String>,
    },
}

#[derive(Debug, Clone, Parser)]
/// Generate hooks for the given shell
pub struct Args {
    #[clap(subcommand)]
    pub command: Option<HookCommands>,

    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<CommandsHooks>,

    #[clap(short = 'E', long, help = "The commands to exclusively enable")]
    enabled: Vec<CommandsHooks>,

    #[clap(short, long, help = "Print hooks for the given shell (ignored if subcommand is used)", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl Args {
    fn print_hook(self, shell: Shell, enabled_hooks: &[CommandsHooks]) {
        match shell {
            Shell::Powershell => {
                print!("function scoop {{ switch ($args[0]) {{ ");

                for command in enabled_hooks {
                    print!(
                        "  '{hook}' {{ return sfsu.exe {command} @($args | Select-Object -Skip 1) }} ",
                        hook = command.hook(),
                        command = command.command()
                    );
                }

                println!("default {{ scoop.ps1 @args }} }} }}");

                println!(
                    "# To add this to your config, add the following line to the end of your PowerShell profile:"
                );
                println!("#     Invoke-Expression (&sfsu hook)");
                println!(
                    "# You can also optionally disable certain hooks via the --disable <COMMAND> flag"
                );
                println!("#     Invoke-Expression (&sfsu hook --disable list)");

                let has_wsl = which::which("wsl").is_ok() || which::which("wsl.exe").is_ok();

                if has_wsl {
                    println!(
                        "# WSL detected: to have the installer add the following to your WSL ~/.bashrc automatically, run `sfsu hook wsl`:"
                    );
                    println!("#   source <(sfsu.exe hook --shell bash)");
                }

                let nu_in_host = which::which("nu").is_ok();
                let mut nu_in_wsl = false;
                if has_wsl {
                    let wsl_cmd = if which::which("wsl.exe").is_ok() {
                        "wsl.exe"
                    } else {
                        "wsl"
                    };
                    nu_in_wsl = SysCommand::new(wsl_cmd)
                        .args(&["command", "-v", "nu"])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                }

                if nu_in_host || nu_in_wsl {
                    println!(
                        "# Nushell is also supported. Run the following command to save it to a file."
                    );
                    println!("#   sfsu hook --shell nu | save -f path/to/some/file.nu");
                    println!(
                        "# Then source it in your config.nu (situated in path $nu.config-path). "
                    );
                    println!("#   source path/to/the/file.nu");
                }
            }
            Shell::Bash | Shell::Zsh => {
                let shell_config = shell.config();
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
                let shell_config = shell.config();
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
    }

    fn install_powershell_hook() -> anyhow::Result<()> {
        let script = crate::scripts::PowershellScript::default_post_install_script();
        let ctx = crate::contexts::User::new()?;
        let runner = script.save(&ctx)?;
        runner.run()?;
        Ok(())
    }

    fn install_wsl_hook(distro: Option<String>) -> anyhow::Result<()> {
        let mut cmd_args = vec![];
        if let Some(ref distro) = distro {
            cmd_args.push("-d");
            cmd_args.push(distro.as_str());
        }

        cmd_args.extend(["--", "bash", "-lc"]);
        let script = "if ! grep -F 'sfsu.exe hook --shell bash' ~/.bashrc >/dev/null 2>&1; then printf \"\\n# >>> sfsu hook >>>\\nsource <(sfsu.exe hook --shell bash)\\n# <<< sfsu hook <<<\\n\" >> ~/.bashrc; fi";
        cmd_args.push(script);

        let wsl_cmd = if which::which("wsl.exe").is_ok() {
            "wsl.exe"
        } else {
            "wsl"
        };

        let output = SysCommand::new(wsl_cmd).args(&cmd_args).output()?;

        if output.status.success() {
            println!("sfsu: ensured WSL bashrc contains sfsu hook");
            Ok(())
        } else {
            anyhow::bail!(
                "Failed to install hook in WSL: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        }
    }
}

impl super::Command for Args {
    async fn runner(self, _: &impl ScoopContext) -> Result<(), anyhow::Error> {
        let enabled_hooks: Vec<CommandsHooks> = {
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

        match self.command.clone() {
            Some(HookCommands::Powershell { print }) if !print => Self::install_powershell_hook()?,
            Some(HookCommands::Bash { print }) if !print => {
                anyhow::bail!(
                    "Automatic installation for Bash is not yet supported on Windows host. Use `sfsu hook bash --print` and add it to your .bashrc manually, or use `sfsu hook wsl` if you are using WSL."
                )
            }
            Some(HookCommands::Zsh { print }) if !print => {
                anyhow::bail!(
                    "Automatic installation for Zsh is not yet supported on Windows host. Use `sfsu hook zsh --print` and add it to your .zshrc manually, or use `sfsu hook wsl` if you are using WSL."
                )
            }
            Some(HookCommands::Nu { print }) if !print => {
                anyhow::bail!(
                    "Automatic installation for Nushell is not yet supported. Use `sfsu hook nu --print` for manual instructions."
                )
            }
            Some(HookCommands::Wsl { distro }) => Self::install_wsl_hook(distro)?,
            Some(cmd) => {
                let shell = match cmd {
                    HookCommands::Powershell { .. } => Shell::Powershell,
                    HookCommands::Bash { .. } => Shell::Bash,
                    HookCommands::Zsh { .. } => Shell::Zsh,
                    HookCommands::Nu { .. } => Shell::Nu,
                    _ => unreachable!(),
                };
                self.print_hook(shell, &enabled_hooks);
            }
            None => {
                let shell = self.shell;
                self.print_hook(shell, &enabled_hooks);
            }
        }

        Ok(())
    }
}
