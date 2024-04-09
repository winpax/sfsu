//! A nicer way to display headers

use std::fmt::Display;

use itertools::Itertools;

#[derive(Debug, Clone)]
#[must_use = "Lazy. Does nothing until consumed"]
/// A nicer way to display headers
pub struct Header<T>(T);

impl<T: Display> Header<T> {
    /// Create a new [`Header`] from the provided value
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Display> Display for Header<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .0
            .to_string()
            .split('_')
            .map(|word| {
                if word.starts_with(|c: char| c.is_uppercase()) {
                    word.to_string()
                } else {
                    let mut word: Vec<char> = word.chars().collect();
                    word[0] = word[0].to_uppercase().nth(0).unwrap();
                    word.into_iter().collect()
                }
            })
            .join(" ");

        write!(f, "{string}")
    }
}
