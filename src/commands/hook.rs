use crate::{contexts::ScoopContext, shell::Shell};
use clap::{Parser, Subcommand};
use quork::traits::list::ListVariants;
use tokio::process::Command as SysCommand;

use super::CommandHooks as CommandsHooks;

#[derive(Debug, Clone, Subcommand)]
pub enum HookCommands {
    #[command(name = "powershell", alias = "pwsh")]
    /// Install the hook for PowerShell automatically
    Powershell {
        #[arg(
            short,
            long,
            help = "Print the hook instead of installing",
            conflicts_with = "uninstall"
        )]
        print: bool,
        #[arg(
            short = 'r',
            long = "rm",
            alias = "uninstall",
            help = "Uninstall the hook",
            conflicts_with = "print"
        )]
        uninstall: bool,
        #[arg(
            short,
            long,
            help = "Install/Uninstall for all users (requires elevation)"
        )]
        system: bool,
    },
    #[command(name = "bash")]
    /// Print the hook for Bash
    Bash {
        #[arg(
            short = 'r',
            long = "rm",
            alias = "uninstall",
            help = "Uninstall the hook"
        )]
        uninstall: bool,
    },
    #[command(name = "zsh")]
    /// Print the hook for Zsh
    Zsh {
        #[arg(
            short = 'r',
            long = "rm",
            alias = "uninstall",
            help = "Uninstall the hook"
        )]
        uninstall: bool,
    },
    #[command(name = "nu")]
    /// Print the hook for Nushell
    Nu {
        #[arg(
            short = 'r',
            long = "rm",
            alias = "uninstall",
            help = "Uninstall the hook"
        )]
        uninstall: bool,
    },
    #[command(name = "wsl")]
    /// Install the hook for WSL automatically
    Wsl {
        /// The WSL distro to install in (defaults to the default distro)
        distro: Option<String>,
        #[arg(
            short = 'r',
            long = "rm",
            alias = "uninstall",
            help = "Uninstall the hook"
        )]
        uninstall: bool,
    },
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
/// Generate hooks for the given shell
pub struct Args {
    #[command(subcommand)]
    pub command: Option<HookCommands>,

    #[arg(short = 'D', long, help = "The commands to disable")]
    disable: Vec<CommandsHooks>,

    #[arg(short = 'E', long, help = "The commands to exclusively enable")]
    enabled: Vec<CommandsHooks>,

    /// Uninstall the hook (top-level flag)
    #[arg(long = "rm", help = "Uninstall the hook")]
    rm: bool,

    #[arg(short, long, help = "Print hooks for the given shell (ignored if subcommand is used)", default_value_t = Shell::Powershell)]
    shell: Shell,
}

impl Args {
    async fn run_command_with_timeout(
        mut command: SysCommand,
        timeout: std::time::Duration,
    ) -> anyhow::Result<std::process::Output> {
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let mut child = command.spawn()?;

        let mut stdout_stream = child.stdout.take().expect("stdout piped");
        let mut stderr_stream = child.stderr.take().expect("stderr piped");

        let stdout_handle = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::new();
            let _ = stdout_stream.read_to_end(&mut buf).await;
            buf
        });
        let stderr_handle = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = Vec::new();
            let _ = stderr_stream.read_to_end(&mut buf).await;
            buf
        });

        match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => {
                let stdout = stdout_handle.await.unwrap_or_default();
                let stderr = stderr_handle.await.unwrap_or_default();
                Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                })
            }
            Ok(Err(err)) => Err(err.into()),
            Err(_) => {
                child.kill().await?;
                anyhow::bail!("Command timed out after {}ms", timeout.as_millis())
            }
        }
    }

    async fn print_hook(self, shell: Shell, enabled_hooks: &[CommandsHooks]) {
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
                    "# To add this to your config, add the following block to the end of your PowerShell profile:"
                );
                println!("# >>> sfsu hook >>>");
                println!("Invoke-Expression (&sfsu hook)");
                println!("# <<< sfsu hook <<<");
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
                        .args(["--", "bash", "-lc", "command -v nu"])
                        .output()
                        .await
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
                    # Add the following block to the end of your ~/.{shell_config} \n\
                    # >>> sfsu hook >>> \n\
                    #   source <(sfsu.exe hook --shell {shell}) \n\
                    # <<< sfsu hook <<<"
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

    fn check_elevation() -> anyhow::Result<()> {
        if !quork::root::is_root().unwrap_or(false) {
            anyhow::bail!(
                "This command requires administrator privileges. Please run as administrator."
            );
        }
        Ok(())
    }

    async fn install_powershell_hook(system: bool) -> anyhow::Result<()> {
        if system {
            Self::check_elevation()?;
        }
        let script = crate::scripts::PowershellScript::default_post_install_script(system);
        let ctx = crate::contexts::User::new()?;
        let runner = script.save(&ctx)?;
        match runner.run() {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                Ok(())
            }
            Err(err) => {
                anyhow::bail!("Failed to run PowerShell installation script: {err}");
            }
        }
    }

    async fn uninstall_powershell_hook(system: bool) -> anyhow::Result<()> {
        if system {
            Self::check_elevation()?;
        }
        let script = crate::scripts::PowershellScript::default_post_uninstall_script(system);
        let ctx = crate::contexts::User::new()?;
        let runner = script.save(&ctx)?;
        match runner.run() {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
                Ok(())
            }
            Err(err) => {
                anyhow::bail!("Failed to run PowerShell uninstallation script: {err}");
            }
        }
    }

    async fn install_wsl_hook(distro: Option<String>) -> anyhow::Result<()> {
        let mut cmd_args = vec![];
        if let Some(ref distro) = distro {
            cmd_args.push("-d");
            cmd_args.push(distro.as_str());
        }

        cmd_args.extend(["--", "bash", "-lc"]);
        let script = "if [ -f ~/.bashrc ] && (grep -q '# >>> sfsu hook >>>' ~/.bashrc || grep -q 'sfsu.exe hook --shell bash' ~/.bashrc); then echo 'sfsu: hook already present'; else printf \"\\n# >>> sfsu hook >>>\\nsource <(sfsu.exe hook --shell bash)\\n# <<< sfsu hook <<<\\n\" >> ~/.bashrc; fi";
        cmd_args.push(script);

        let wsl_cmd = if which::which("wsl.exe").is_ok() {
            "wsl.exe"
        } else {
            "wsl"
        };

        let mut cmd = SysCommand::new(wsl_cmd);
        cmd.args(&cmd_args);
        let output = Self::run_command_with_timeout(cmd, std::time::Duration::from_secs(30)).await;

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("already present") {
                    println!("sfsu: Hook already present in WSL");
                } else {
                    println!("sfsu: Ensured WSL bashrc contains sfsu hook");
                }
                Ok(())
            }
            Ok(output) => {
                anyhow::bail!(
                    "Failed to install hook in WSL (exit code {}): {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                )
            }
            Err(err) => {
                anyhow::bail!("Failed to execute WSL command: {err}");
            }
        }
    }

    async fn uninstall_wsl_hook(distro: Option<String>) -> anyhow::Result<()> {
        let mut cmd_args = vec![];
        if let Some(ref distro) = distro {
            cmd_args.push("-d");
            cmd_args.push(distro.as_str());
        }

        cmd_args.extend(["--", "bash", "-lc"]);
        // Try block removal first, fallback to line removal
        let script = "if [ -f ~/.bashrc ] && grep -q '# >>> sfsu hook >>>' ~/.bashrc; then sed -i '/# >>> sfsu hook >>>/,/# <<< sfsu hook <<</d' ~/.bashrc && echo 'removed block'; elif [ -f ~/.bashrc ] && grep -q 'sfsu.exe hook --shell bash' ~/.bashrc; then sed -i '/sfsu.exe hook --shell bash/d' ~/.bashrc && echo 'removed line'; fi";
        cmd_args.push(script);

        let wsl_cmd = if which::which("wsl.exe").is_ok() {
            "wsl.exe"
        } else {
            "wsl"
        };

        let mut cmd = SysCommand::new(wsl_cmd);
        cmd.args(&cmd_args);
        let output = Self::run_command_with_timeout(cmd, std::time::Duration::from_secs(30)).await;

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("removed") {
                    println!("sfsu: Removed hook from WSL bashrc");
                } else {
                    println!("sfsu: Hook markers or command not found in WSL");
                }
                Ok(())
            }
            Ok(output) => {
                anyhow::bail!(
                    "Failed to uninstall hook from WSL (exit code {}): {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                )
            }
            Err(err) => {
                anyhow::bail!("Failed to execute WSL command: {err}");
            }
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
            Some(HookCommands::Powershell {
                print,
                uninstall,
                system,
            }) => {
                if uninstall || self.rm {
                    Self::uninstall_powershell_hook(system).await?
                } else if !print {
                    Self::install_powershell_hook(system).await?
                } else {
                    self.print_hook(Shell::Powershell, &enabled_hooks).await
                }
            }
            Some(HookCommands::Bash { uninstall }) => {
                if uninstall || self.rm {
                    anyhow::bail!(
                        "Automatic uninstallation for Bash is not yet supported on Windows host. Please remove the hook from your .bashrc manually."
                    )
                } else {
                    self.print_hook(Shell::Bash, &enabled_hooks).await
                }
            }
            Some(HookCommands::Zsh { uninstall }) => {
                if uninstall || self.rm {
                    anyhow::bail!(
                        "Automatic uninstallation for Zsh is not yet supported on Windows host. Please remove the hook from your .zshrc manually."
                    )
                } else {
                    self.print_hook(Shell::Zsh, &enabled_hooks).await
                }
            }
            Some(HookCommands::Nu { uninstall }) => {
                if uninstall || self.rm {
                    anyhow::bail!(
                        "Automatic uninstallation for Nushell is not yet supported. Please remove the hook from your config.nu manually."
                    )
                } else {
                    self.print_hook(Shell::Nu, &enabled_hooks).await
                }
            }
            Some(HookCommands::Wsl { distro, uninstall }) => {
                if uninstall || self.rm {
                    Self::uninstall_wsl_hook(distro).await?
                } else {
                    Self::install_wsl_hook(distro).await?
                }
            }
            None => {
                let shell = self.shell;
                self.print_hook(shell, &enabled_hooks).await;
            }
        }

        Ok(())
    }
}
