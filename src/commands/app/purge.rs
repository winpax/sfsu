use clap::Parser;
use dialoguer::Confirm;
use sprinkles::{config, contexts::ScoopContext, packages::reference::package};

use crate::output::colours::{eprintln_yellow, yellow};

#[derive(Debug, Clone, Parser)]
/// Purge package's persist folder
pub struct Args {
    #[clap(help = "The package to purge")]
    app: package::Reference,

    #[clap(from_global)]
    assume_yes: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: &impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let app = self.app.first(ctx).unwrap();
        let persist_path = ctx.persist_path().join(unsafe { app.name() });

        if !persist_path.exists() {
            eprintln_yellow!("Persist folder does not exist for {}", unsafe {
                app.name()
            });
            return Ok(());
        }

        if !self.assume_yes
            && !Confirm::new()
                .with_prompt(
                    yellow!(
                        "Are you sure you want to purge the persist folder for \"{}\" ({})?",
                        unsafe { app.name() },
                        persist_path.display()
                    )
                    .to_string(),
                )
                .default(false)
                .interact()?
        {
            return Ok(());
        }

        if app.is_installed(ctx, None)
            && !self.assume_yes
            && !Confirm::new()
                .with_prompt(
                    yellow!(
                        "\"{}\" is installed. This could cause issues when running the app. Are you sure you want to continue?",
                        unsafe { app.name() },
                    )
                    .to_string(),
                )
                .default(false)
                .interact()?
        {
            return Ok(());
        }

        eprintln_yellow!("Purging persist folder for {}", unsafe { app.name() });

        std::fs::remove_dir_all(persist_path)?;

        Ok(())
    }
}
