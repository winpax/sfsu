use std::{fmt::Display, rc::Rc};

use rayon::prelude::*;
use serde_json::Value;

#[must_use = "Structured is lazy, and only takes effect when used in formatting"]
/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured<'a> {
    headers: &'a [&'a str],
    values: &'a [Value],
}

impl<'a> Structured<'a> {
    /// Construct a new [`Structured`] formatter
    ///
    /// # Panics
    /// - If the length of headers is not equal to the length of values
    /// - If the values provided are not objects
    pub fn new(headers: &'a [&'a str], values: &'a [Value]) -> Self {
        assert_eq!(
            headers.len(),
            // TODO: Do not panic here
            values[0].as_object().unwrap().keys().len(),
            "The number of column headers must match quantity data for said columns"
        );
        Structured { headers, values }
    }
}

impl<'a> Display for Structured<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = {
            let mut data = vec![];
            self.values
                .par_iter()
                // TODO: Do not panic here
                .map(|row| row.as_object().expect("object"))
                .collect_into_vec(&mut data);

            data
        };

        // Headers are the display headers, with the first letter capitalised
        // Access is the name used to access the value on the object
        // let headers = {
        //     // TODO: Do not panic here
        //     let first = data.first().expect("non-empty data");
        //     first
        //         .keys()
        //         .map(|v| {
        //             let mut chars = v.chars();

        //             match chars.next() {
        //                 // Should be unreachable
        //                 // TODO: Handle this case better
        //                 None => String::new(),
        //                 Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        //             }
        //         })
        //         .collect::<Vec<_>>()
        // };

        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(self.headers.iter().map(|s| s.len()));

            v
        };

        // TODO: Imeplement max length with truncation
        let access_lengths: Vec<usize> =
            data.iter().fold(vec![0; self.headers.len()], |base, row| {
                // TODO: Simultaneous iterators

                self.headers
                    .iter()
                    .enumerate()
                    .map(|(i, header)| {
                        let element = row
                            .get(*header)
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();

                        let mut contestants = contestants.clone();
                        contestants.push(base[i]);
                        contestants.push(element.to_string().len());

                        *contestants.iter().max().unwrap()
                    })
                    .collect()
            });

        dbg!(&access_lengths);

        for (i, header) in self.headers.iter().enumerate() {
            let header_size = access_lengths[i];
            write!(f, "{header:header_size$} | ")?;
        }

        // Enter new row
        writeln!(f)?;

        for row in &data {
            for (i, header) in self.headers.iter().enumerate() {
                let value_size = access_lengths[i];
                let element = row
                    .get(*header)
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                write!(f, "{element:value_size$} | ")?;
            }

            // Enter new row
            writeln!(f)?;
        }

        Ok(())
    }
}
