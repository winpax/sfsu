use clap::ValueEnum;
use strum::Display;

#[derive(Debug, Default, ValueEnum, Copy, Clone, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Shell {
    #[default]
    Powershell,
    Bash,
    Zsh,
    Nu,
}

impl Shell {
    #[must_use]
    pub fn config(self) -> ShellConfig {
        ShellConfig::new(self)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct ShellConfig(Shell);

impl ShellConfig {
    #[must_use]
    pub fn new(shell: Shell) -> Self {
        Self(shell)
    }

    #[must_use]
    pub fn path(self) -> &'static str {
        match self.0 {
            Shell::Powershell => "$PROFILE",
            Shell::Bash => "bashrc",
            Shell::Zsh => "zshrc",
            Shell::Nu => "$nu.config-path",
        }
    }
}

impl std::fmt::Display for ShellConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path())
    }
}
