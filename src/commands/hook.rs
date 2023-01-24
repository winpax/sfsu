use clap::Parser;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(short = 'D', long, help = "The commands to disable")]
    disable: Vec<super::CommandsRaw>,
}

// TODO: Add function to generate hooks

impl super::Command for Args {
    type Error = anyhow::Error;

    fn run(self) -> Result<(), Self::Error> {
        print!("function scoop {{ ");

        let enabled_hooks: Vec<super::CommandsRaw> = super::CommandsRaw::iter()
            .filter(|variant| !self.disable.contains(variant))
            .collect();

        for command in enabled_hooks {
            let command_name: String = command.to_string();
            print!(
                "if ($args[0] -eq '{command_name}') {{ sfsu.exe {command_name} @($args | Select-Object -Skip 1) }} else"
            );
        }

        print!(" {{ scoop.ps1 @args }} }}");

        Ok(())
    }
}
