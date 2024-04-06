use std::fmt::Display;

use itertools::Itertools;
use serde_json::{Map, Value};

use super::wrappers::header::Header;

pub mod vertical;

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
    // but given we are just forwarding what is passed to `Structured`,
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

fn print_headers(
    f: &mut std::fmt::Formatter<'_>,
    headers: &[&String],
    max_length: Option<usize>,
    access_lengths: &[usize],
) -> std::fmt::Result {
    #[cfg(feature = "v2")]
    {
        use colored::Colorize as _;

        let header_lengths = headers
            .iter()
            .enumerate()
            .map(|(i, header)| -> Result<usize, std::fmt::Error> {
                let header_size = access_lengths[i];

                let truncated = OptionalTruncate::new(Header::new(header))
                    .with_length(max_length)
                    .to_string();
                write!(f, "{:header_size$} ", truncated.bright_green())?;

                Ok(truncated.len())
            })
            .collect::<Result<Vec<_>, _>>()?;

        writeln!(f)?;

        for (i, length) in header_lengths.into_iter().enumerate() {
            let header_size = access_lengths[i];

            let underscores = "-".repeat(length);

            write!(f, "{:header_size$} ", underscores.bright_green())?;
        }
    }

    #[cfg(not(feature = "v2"))]
    for (i, header) in headers.iter().enumerate() {
        let header_size = access_lengths[i];

        let truncated = OptionalTruncate::new(Header::new(header)).with_length(max_length);
        write!(f, "{truncated:header_size$} | ")?;
    }

    Ok(())
}

#[must_use = "Structured is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured<'a> {
    objects: Vec<&'a Map<String, Value>>,
    max_length: Option<usize>,
}

impl<'a> Structured<'a> {
    /// Construct a new [`Structured`] formatter
    ///
    /// # Panics
    /// - If the length of headers is not equal to the length of values
    /// - If the values provided are not objects
    pub fn new(values: &'a [Value]) -> Self {
        let objects = values
            .iter()
            .map(|v| v.as_object().expect("object"))
            .collect::<Vec<_>>();

        Structured {
            objects,
            max_length: None,
        }
    }

    /// Add a max length to the [`Structured`] formatter
    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);

        self
    }
}

impl<'a> Display for Structured<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = self.objects[0].keys().collect_vec();

        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(headers.iter().map(|s| s.len()));

            v
        };

        // TODO: Imeplement max length with truncation
        let access_lengths: Vec<usize> =
            self.objects
                .iter()
                .fold(vec![0; headers.len()], |base, row| {
                    // TODO: Simultaneous iterators

                    headers
                        .iter()
                        .enumerate()
                        .map(|(i, header)| {
                            let element = row
                                .get(&heck::AsSnakeCase(header).to_string())
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();

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

        print_headers(f, &headers, self.max_length, &access_lengths)?;

        // Enter new row
        writeln!(f)?;

        for row in &self.objects {
            for (i, header) in headers.iter().enumerate() {
                let value_size = access_lengths[i];

                let element = row
                    .get(&heck::AsSnakeCase(header).to_string())
                    .and_then(|v| {
                        if let Some(s) = v.as_str() {
                            Some(s.to_string())
                        } else {
                            v.as_array().map(|array| {
                                array
                                    .iter()
                                    .map(|v| {
                                        v.as_str()
                                            .map(std::string::ToString::to_string)
                                            .unwrap_or_default()
                                    })
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            })
                        }
                    })
                    .unwrap_or_default();

                let with_suffix = match element.len().cmp(&value_size) {
                    std::cmp::Ordering::Greater => format!("{}...", &element[0..value_size - 3]),
                    std::cmp::Ordering::Equal => element.to_string(),
                    std::cmp::Ordering::Less => format!("{element:value_size$}"),
                };

                #[cfg(feature = "v2")]
                write!(f, "{with_suffix} ")?;
                #[cfg(not(feature = "v2"))]
                write!(f, "{with_suffix} | ")?;
            }

            // Enter new row
            writeln!(f)?;
        }

        Ok(())
    }
}
