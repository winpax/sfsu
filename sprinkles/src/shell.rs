//! Shell related utilities

use strum::Display;

#[derive(Debug, Default, Copy, Clone, Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[allow(missing_docs)]
/// A supported shell
pub enum Shell {
    #[default]
    Powershell,
    Bash,
    Zsh,
    Nu,
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for Shell {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Powershell, Self::Bash, Self::Zsh, Self::Nu]
    }
    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Powershell => Some(clap::builder::PossibleValue::new("powershell")),
            Self::Bash => Some(clap::builder::PossibleValue::new("bash")),
            Self::Zsh => Some(clap::builder::PossibleValue::new("zsh")),
            Self::Nu => Some(clap::builder::PossibleValue::new("nu")),
        }
    }
}

impl Shell {
    #[must_use]
    /// Get the shell config path
    pub fn config(self) -> ShellConfig {
        ShellConfig::new(self)
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
/// The shell config path
pub struct ShellConfig(Shell);

impl ShellConfig {
    #[must_use]
    /// Create a new shell config path from the provided [`Shell`]
    pub fn new(shell: Shell) -> Self {
        Self(shell)
    }

    #[must_use]
    /// Get the shell config path
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
