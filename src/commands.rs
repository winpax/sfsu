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
