//! Calm Panic helpers
//!
//! This module provides ways to exit the program with an error message, without panicking

use std::fmt::{Debug, Display};

/// Trait for unwrapping `Result` and `Option` without panicking
pub trait CalmUnwrap<T> {
    /// Unwrap the value, or panic with a message
    fn calm_unwrap(self) -> T;

    /// Unwrap the value, or panic with a message
    fn calm_expect(self, msg: impl AsRef<str>) -> T;
}

impl<T, E: Debug> CalmUnwrap<T> for Result<T, E> {
    fn calm_unwrap(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => __calm_panic(format!("`Result` had error value: {e:?}")),
        }
    }

    fn calm_expect(self, msg: impl AsRef<str>) -> T {
        match self {
            Ok(v) => v,
            Err(e) => __calm_panic(format!("{}. {e:?}", msg.as_ref())),
        }
    }
}

impl<T> CalmUnwrap<T> for Option<T> {
    fn calm_unwrap(self) -> T {
        match self {
            Some(v) => v,
            None => __calm_panic("Option had no value"),
        }
    }

    fn calm_expect(self, msg: impl AsRef<str>) -> T {
        match self {
            Some(v) => v,
            None => __calm_panic(msg.as_ref()),
        }
    }
}

#[doc(hidden)]
// TODO: Add option to not pass any message
pub fn __calm_panic(msg: impl Display) -> ! {
    use owo_colors::OwoColorize;
    eprintln!("{}", msg.to_string().red());
    std::process::exit(1);
}

#[macro_export]
/// Abandon the current execution with a message
macro_rules! abandon {
    ($($t:tt)*) => {
        $crate::calm_panic::__calm_panic(format!($($t)*))
    };
}

pub use abandon;
