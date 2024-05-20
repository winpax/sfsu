//! Installer helpers

use crate::{config, contexts::ScoopContext, packages::manifest::Installer, scripts};

use super::models::manifest::{InstallerRunner, TOrArrayOfTs};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
/// Errors that can occur when running an installer
pub enum Error {
    #[error("Installer I/O error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Installer script error: {0}")]
    Scripts(#[from] scripts::Error),
}

/// Installer result type
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Installer {
    #[must_use]
    /// Get the installer runner
    pub fn get_runner(&self) -> Option<InstallerRunner> {
        self.script
            .clone()
            .map(InstallerRunner::Script)
            .or_else(|| self.file.clone().map(InstallerRunner::File))
    }

    /// Run the installer
    ///
    /// # Errors
    /// - If the installer could not be run
    pub async fn run(&self, ctx: &impl ScoopContext<config::Scoop>) -> Result<()> {
        let runner = self.get_runner();
        let args = self.args.clone().map(TOrArrayOfTs::to_vec);

        match runner {
            Some(InstallerRunner::File(file)) => {
                let mut command = std::process::Command::new(file);

                if let Some(ref args) = args {
                    command.args(args)
                } else {
                    &mut command
                }
                .spawn()?
                .wait()?;
            }
            Some(InstallerRunner::Script(script)) => script.save(ctx)?.run().await?,
            None => {}
        }

        Ok(())
    }
}
