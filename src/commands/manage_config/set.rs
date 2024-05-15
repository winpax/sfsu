use clap::Parser;
use serde_json::{Number, Value};
use sprinkles::{config, contexts::ScoopContext};

#[derive(Debug, Clone, Parser)]
pub struct Args {
    #[clap(help = "The key to set")]
    field: String,

    #[clap(help = "The value to set")]
    value: String,
}

impl super::Command for Args {
    async fn runner(self, mut ctx: impl ScoopContext<config::Scoop>) -> anyhow::Result<()> {
        let mut config_manager = super::management::ManageConfig::new(ctx.config_mut());

        config_manager.set(&self.field, string_to_value(&self.value))?;

        println!("'{}' has been set to '{}'", self.field, self.value);

        Ok(())
    }
}

fn string_to_value(string: impl AsRef<str>) -> Value {
    match string.as_ref() {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    };

    if let Ok(number) = string.as_ref().parse::<Number>() {
        return Value::Number(number);
    }

    Value::String(string.as_ref().into())
}
