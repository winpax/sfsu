use clap::{Parser, Subcommand};
use sfsu_derive::Runnable;
use sprinkles::{config, contexts::ScoopContext};

use crate::output::structured;

use super::{Command, CommandRunner};

pub mod get;
pub mod remove;
pub mod set;

#[derive(Debug, Clone, Subcommand, Runnable)]
pub enum Commands {
    #[clap(alias = "rm")]
    /// Remove a value from the config
    Remove(remove::Args),
    /// Set a value in the config
    Set(set::Args),
    /// Get a value from the config
    Get(get::Args),
}

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(subcommand)]
    command: Option<Commands>,

    #[clap(from_global)]
    json: bool,
}

impl super::Command for Args {
    async fn runner(self, ctx: impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let ctx = scopeguard::guard(ctx, |ctx| {
            ctx.config().save().expect("could not save config");
        });

        if let Some(command) = self.command {
            return command.run(ctx.clone()).await;
        }

        if self.json {
            let config = ctx.config();

            println!("{}", serde_json::to_string_pretty(&config)?);
        } else {
            let config = ctx.config();
            let mut vtable = structured::vertical::VTable::new(config);
            vtable.snake_case_headers();

            println!("{vtable}");
        }

        todo!("print config");
    }
}

mod management {
    use anyhow::Context;
    use serde_json::Value;

    use sprinkles::config;

    #[derive(Debug)]
    pub struct ManageConfig<'a> {
        config: &'a mut config::Scoop,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Operation {
        Remove(String),
        Set { field: String, value: Value },
    }

    impl<'a> ManageConfig<'a> {
        pub fn new(config: &'a mut config::Scoop) -> Self {
            Self { config }
        }

        pub fn remove(&mut self, field: impl Into<String>) -> anyhow::Result<()> {
            self.execute(Operation::Remove(field.into()))
        }

        pub fn set(&mut self, field: impl Into<String>, value: Value) -> anyhow::Result<()> {
            self.execute(Operation::Set {
                field: field.into(),
                value,
            })
        }

        pub fn get(&self, field: impl Into<String>) -> anyhow::Result<Value> {
            let value = self.config.to_object()?;

            let object = value.as_object().context("invalid json object")?;

            object
                .get(&field.into())
                .context("field not found")
                .cloned()
        }

        fn execute(&mut self, operation: Operation) -> anyhow::Result<()> {
            let mut value = self.config.to_object()?;

            let object = value.as_object_mut().context("invalid json object")?;

            match operation {
                Operation::Remove(field) => object.remove(&field),
                Operation::Set { field, value } => object.insert(field.to_string(), value),
            };

            let config = serde_json::from_value(value)?;

            *self.config = config;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_remove() {
            let mut config = config::Scoop::load().unwrap();

            config.scoop_branch = "develop".into();

            let mut manage_config = ManageConfig::new(&mut config);

            manage_config.remove("scoop_branch").unwrap();

            // Master used here is the default value
            assert_eq!(config.scoop_branch, "master".into());
        }

        #[test]
        fn test_set() {
            let mut config = config::Scoop::load().unwrap();

            config.scoop_branch = "master".into();

            assert_eq!(config.scoop_branch, "master".into());

            let mut manage_config = ManageConfig::new(&mut config);

            manage_config.set("scoop_branch", "develop".into()).unwrap();

            assert_eq!(config.scoop_branch, "develop".into());
        }
    }
}
