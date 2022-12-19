pub mod hook;
pub mod list;
pub mod search;

pub trait Command {
    type Error;

    fn run(self) -> Result<(), Self::Error>;
}
