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

        config_manager.set(self.field, string_to_value(self.value))?;

        ctx.config().save()?;

        Ok(())
    }
}

fn string_to_value(string: String) -> Value {
    match string.as_str() {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    };

    if let Ok(number) = string.parse::<Number>() {
        return Value::Number(number);
    }

    Value::String(string)
}
