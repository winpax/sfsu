use std::fmt::Display;

use itertools::Itertools;
use serde_json::{Map, Value};

use crate::{output::wrappers::header::Header, SimIter};

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

    // Generally length would not be passed as an option,
    // but given we are just forwarding what is passed to `VTable`,
    // it works better here
    pub fn with_length(self, length: Option<usize>) -> Self {
        Self { length, ..self }
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

#[must_use = "VTable is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct VTable {
    object: Map<String, Value>,
    max_length: Option<usize>,
}

impl VTable {
    // /// Construct a new [`VTable`] formatter
    // ///
    // /// # Panics
    // /// - If the length of headers is not equal to the length of values
    // /// - If the values provided are not objects
    // pub fn new(headers: &'a [H], values: &'a [V]) -> Self {
    //     assert_eq!(
    //         headers.len(),
    //         // TODO: Do not panic here
    //         values.len(),
    //         "The number of column headers must match quantity data for said columns"
    //     );
    //     Self {
    //         headers,
    //         values,
    //         max_length: None,
    //     }
    // }

    pub fn from_value(value: Value) -> Self {
        let object = value.as_object().expect("Value must be an object").clone();

        Self {
            object,
            max_length: None,
        }
    }

    /// Add a max length to the [`VTable`] formatter
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);

        self
    }
}

impl Display for VTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = self.object.keys().collect_vec();
        let values = self.object.values().collect_vec();

        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(headers.iter().map(|s| s.len()));

            v
        };

        let header_lengths: Vec<usize> =
            headers
                .iter()
                .fold(vec![0; headers.len()], |base, element| {
                    // TODO: Simultaneous iterators

                    headers
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let mut contestants = contestants.clone();
                            contestants.push(base[i]);
                            contestants.push(
                                OptionalTruncate::new(element)
                                    .with_length(self.max_length)
                                    // TODO: Fix suffix
                                    .with_suffix("...")
                                    .to_string()
                                    .len(),
                            );

                            *contestants.iter().max().unwrap()
                        })
                        .collect()
                });

        let iters = SimIter(headers.iter(), values.iter()).enumerate();

        for (i, (header, element)) in iters {
            let header_size = header_lengths[i];

            let truncated =
                OptionalTruncate::new(Header::new(header).to_string()).with_length(self.max_length);

            let element = if let Some(element) = element.as_str() {
                element.to_owned()
            } else if let Some(array) = element.as_array() {
                array
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map(std::string::ToString::to_string)
                            .unwrap_or_default()
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            } else {
                element.to_string()
            };

            writeln!(f, "{truncated:header_size$} : {element}")?;
        }

        Ok(())
    }
}
