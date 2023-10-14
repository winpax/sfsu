use std::fmt::Display;

use strum::IntoEnumIterator;

pub fn enum_to_string<T: IntoEnumIterator + Display>() -> String {
    T::iter().map(|v| v.to_string()).collect::<String>()
}
