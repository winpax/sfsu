use std::fmt::Display;

#[derive(Debug)]
#[must_use = "OptionalTruncate is lazy, and only takes effect when used in formatting"]
pub struct OptionalTruncate<T> {
    data: T,
    length: Option<usize>,
    suffix: Option<&'static str>,
}

impl<T> OptionalTruncate<T> {
    /// Construct a new [`OptionalTruncate`] from the provided data
    pub fn new(data: T) -> Self {
        Self {
            data,
            length: None,
            suffix: None,
        }
    }

    pub fn with_suffix(self, suffix: &'static str) -> Self {
        Self {
            suffix: Some(suffix),
            ..self
        }
    }
}

impl<T: Display> Display for OptionalTruncate<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(length) = self.length {
            use quork::truncate::Truncate;

            let mut truncation = Truncate::new(&self.data, length);

            if let Some(ref suffix) = self.suffix {
                truncation = truncation.with_suffix(suffix);
            }

            truncation.to_string();

            truncation.fmt(f)
        } else {
            self.data.fmt(f)
        }
    }
}
