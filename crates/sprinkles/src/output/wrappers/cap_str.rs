use std::fmt::Display;

#[macro_export]
macro_rules! wrap_str {
    ($v:literal) => {
        CapitalizedStr::new($v)
    };
}

pub use wrap_str;

#[derive(Debug, Clone)]
#[must_use = "Lazy. Does nothing until consumed"]
pub struct CapitalizedStr<T>(T);

impl<T: Display> CapitalizedStr<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Display> Display for CapitalizedStr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = self.0.to_string();

        if !string.starts_with(|c: char| c.is_uppercase()) {
            let mut v: Vec<char> = string.chars().collect();
            v[0] = v[0].to_uppercase().nth(0).unwrap();
            string = v.into_iter().collect();
        }

        write!(f, "{string}")
    }
}
