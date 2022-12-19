use clap::Subcommand;

pub mod hook;
pub mod list;
pub mod search;

pub trait Command {
    type Error;

    /// Execute the command
    ///
    /// # Errors
    /// - May run into an error
    fn run(self) -> Result<(), Self::Error>;
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Search(search::Args),
    List(list::Args),
    Hook(hook::Args),
}

impl Command for Commands {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        // TODO: Find a way to unpack inner value without match statement
        match self {
            Commands::Search(args) => args.run()?,
            Commands::List(args) => args.run()?,
            Commands::Hook(args) => args.run()?,
        }

        Ok(())
    }
}
