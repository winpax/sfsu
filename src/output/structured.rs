use std::{fmt::Display, rc::Rc};

use rayon::prelude::*;
use serde_json::Value;

/// A table of data
///
/// Takes a single named lifetime, given that this is intended
/// to be constructed and used within the same function.
pub struct Structured<'a>(&'a [Value]);

impl<'a> Display for Structured<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let row_access: Vec<String> = self
        //     .headers
        //     .iter()
        //     .map(|header| header.to_lowercase())
        //     .collect();

        let data = {
            let mut data = vec![];
            self.0
                .par_iter()
                // TODO: Do not panic here
                .map(|row| row.as_object().expect("object"))
                .collect_into_vec(&mut data);

            data
        };

        // Headers are the display headers, with the first letter capitalised
        // Access is the name used to access the value on the object
        let headers = {
            // TODO: Do not panic here
            let first = data.first().expect("non-empty data");
            first
                .keys()
                .map(|v| {
                    let mut chars = v.chars();

                    match chars.next() {
                        // Should be unreachable
                        // TODO: Handle this case better
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
        };

        let contestants = {
            // TODO: Make this dynamic largest header
            let default_width = "Updated".len();

            let mut v = vec![default_width];
            v.extend(headers.iter().map(|h| h.len()));

            v
        };

        // TODO: Imeplement max length with truncation
        let access_lengths: Vec<usize> =
            data.iter().fold(vec![0; headers.len()], |base, current| {
                // TODO: Simultaneous iterators
                current
                    .values()
                    .map(|element| {
                        let mut contestants = contestants.clone();
                        contestants.push(element.to_string().len());

                        *contestants.iter().max().unwrap()
                    })
                    .collect()
            });

        dbg!(&access_lengths);

        for (i, header) in headers.iter().enumerate() {
            let header_size = access_lengths[i];
            write!(f, "{header:header_size$}")?;
        }

        for row in &data {
            for (i, element) in row.values().enumerate() {
                let value_size = access_lengths[i];
                write!(f, "{element:value_size$}")?;
            }
        }

        todo!()
    }
}
