use std::fmt::{Debug, Display};

pub trait CalmUnwrap<T> {
    fn calm_unwrap(self) -> T;

    fn calm_expect(self, msg: impl AsRef<str>) -> T;
}

impl<T, E: Debug> CalmUnwrap<T> for Result<T, E> {
    fn calm_unwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => calm_panic(format!("`Result` had error value: {e:?}")),
        }
    }

    fn calm_expect(self, msg: impl AsRef<str>) -> T {
        match self {
            Ok(v) => v,
            Err(e) => calm_panic(format!("{}. {e:?}", msg.as_ref())),
        }
    }
}

impl<T> CalmUnwrap<T> for Option<T> {
    fn calm_unwrap(self) -> T {
        match self {
            Some(v) => v,
            None => calm_panic("Option had no value"),
        }
    }

    fn calm_expect(self, msg: impl AsRef<str>) -> T {
        match self {
            Some(v) => v,
            None => calm_panic(msg.as_ref()),
        }
    }
}

pub fn calm_panic(msg: impl Display) -> ! {
    use colored::Colorize as _;
    eprintln!("{}", msg.to_string().red());
    std::process::exit(1);
}
