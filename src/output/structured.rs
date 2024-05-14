//! Structured output for the CLI

use std::fmt::Display;

use itertools::Itertools;
use serde::Serialize;
use serde_json::{Map, Value};

use sprinkles::{hacks::inline_const::inline_const, wrappers::header::Header};

use super::truncate::OptionalTruncate;

pub mod vertical;

const WALL: &str = " | ";
const SUFFIX: &str = "...";

struct TruncateOrPad(String, usize);

impl Display for TruncateOrPad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let length = self.1 - WALL.len();
        if self.0.len() > length {
            write!(
                f,
                "{}{SUFFIX}",
                &self.0[0..length.checked_sub(3).unwrap_or_default()]
            )
        } else {
            write!(f, "{:width$}", self.0, width = length)
        }
    }
}

#[deprecated]
#[allow(dead_code, unused_variables)]
fn print_headers(
    f: &mut std::fmt::Formatter<'_>,
    headers: &[&String],
    max_length: Option<usize>,
    access_lengths: &[usize],
) -> std::fmt::Result {
    #[cfg(feature = "v2")]
    {
        use owo_colors::OwoColorize;

        let header_lengths = headers
            .iter()
            .enumerate()
            .map(|(i, header)| -> Result<usize, std::fmt::Error> {
                let header_size = access_lengths[i];

                let truncated = OptionalTruncate::new(Header::new(header))
                    .with_length(max_length)
                    .to_string();
                write!(
                    f,
                    "{:header_size$}{WALL}",
                    console::style(truncated).green().bright()
                )?;

                Ok(truncated.len())
            })
            .collect::<Result<Vec<_>, _>>()?;

        writeln!(f)?;

        for (i, length) in header_lengths.into_iter().enumerate() {
            let header_size = access_lengths[i];

            let underscores = "-".repeat(length);

            write!(
                f,
                "{:header_size$}{WALL}",
                console::style(underscores).green().bright()
            )?;
        }
    }

    #[cfg(not(feature = "v2"))]
    for (i, header) in headers.iter().enumerate() {
        let header_size = access_lengths[i];

        let truncated = TruncateOrPad(Header::new(header).to_string(), header_size).to_string();
        write!(f, "{truncated}{WALL}")?;
    }

    Ok(())
}

#[must_use = "Structured is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured {
    objects: Vec<Map<String, Value>>,
    max_length: Option<usize>,
}

impl Structured {
    /// Construct a new [`Structured`] formatter
    ///
    /// # Panics
    /// - If the length of headers is not equal to the length of values
    /// - If the values provided are not objects
    pub fn new(values: &[impl Serialize]) -> Self {
        let objects = values
            .iter()
            .map(|v| {
                let value = serde_json::to_value(v).expect("valid value");

                let object = value.as_object().expect("object").clone();

                object
            })
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

impl Display for Structured {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let headers = self.objects[0].keys().collect_vec();

        let contestants = {
            let default_width = headers
                .iter()
                .map(|header| header.len())
                .max()
                .unwrap_or(inline_const!(usize "Updated".len()));

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
                                    // TODO: Fix suffix
                                    .with_suffix(SUFFIX)
                                    .to_string()
                                    .len()
                                    + WALL.len(),
                            );

                            contestants.into_iter().max().unwrap()
                        })
                        .collect()
                });

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let evened_access_lengths = {
            let term_columns: f64 = console::Term::stdout().size().1.into();
            let total = access_lengths.iter().sum::<usize>() as f64;
            let percents = access_lengths.iter().map(|s| ((*s) as f64) / total);
            let even_parts = percents.map(|p| (p * term_columns).floor() as usize);

            even_parts.collect::<Vec<_>>()
        };

        let access_lengths = evened_access_lengths;

        for (i, header) in headers.iter().enumerate() {
            let header_size = access_lengths[i];

            let truncated = TruncateOrPad(Header::new(header).to_string(), header_size).to_string();
            write!(f, "{truncated}{WALL}")?;
        }

        // Enter new row
        writeln!(f)?;

        for row in &self.objects {
            for (i, header) in headers.iter().enumerate() {
                let value_size = access_lengths[i];

                let element = row
                    .get(&heck::AsSnakeCase(header).to_string())
                    .and_then(|v| match v {
                        Value::Null => None,
                        Value::Bool(bool) => Some(bool.to_string()),
                        Value::Number(number) => Some(number.to_string()),
                        Value::String(string) => Some(string.to_string()),
                        Value::Array(array) => Some(
                            array
                                .iter()
                                .map(|v| {
                                    v.as_str()
                                        .map(std::string::ToString::to_string)
                                        .unwrap_or_default()
                                })
                                .collect::<Vec<String>>()
                                .join(", "),
                        ),
                        Value::Object(_) => panic!("Objects not supported within other objects"),
                    })
                    .unwrap_or_default();

                let with_suffix = TruncateOrPad(element, value_size);

                #[cfg(feature = "v2")]
                write!(f, "{with_suffix}{WALL}")?;
                #[cfg(not(feature = "v2"))]
                write!(f, "{with_suffix}{WALL}")?;
            }

            // Enter new row
            writeln!(f)?;
        }

        Ok(())
    }
}
