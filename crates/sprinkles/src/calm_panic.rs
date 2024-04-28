//! Calm Panic helpers
//!
//! This module provides ways to exit the program with an error message, without panicking

use std::fmt::Debug;

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
            Err(e) => abandon!("`Result` had error value: {e:?}"),
        }
    }

    fn calm_expect(self, msg: impl AsRef<str>) -> T {
        match self {
            Ok(v) => v,
            Err(e) => abandon!("{}. {e:?}", msg.as_ref()),
        }
    }
}

impl<T> CalmUnwrap<T> for Option<T> {
    fn calm_unwrap(self) -> T {
        self.unwrap_or_else(|| abandon!("Option had no value"))
    }

    fn calm_expect(self, msg: impl AsRef<str>) -> T {
        self.unwrap_or_else(|| abandon!("{}", msg.as_ref()))
    }
}

#[macro_export]
/// Abandon the current execution with a message
macro_rules! abandon {
    () => {
        abandon!("Abandoned execution");
    };

    ($($t:tt)*) => {{
        use $crate::output::colours::red;
        red!($($t)*);
        std::process::exit(1);
    }};
}

pub use abandon;
