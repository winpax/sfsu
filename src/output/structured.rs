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

        let access_lengths = headers
            .iter()
            .fold(vec![0; headers.len()], |base, current| {
                // Checks for the largest size out of the previous one, the current one and the section title
                // Note that all widths use "Updated" as it is the longest section title
                // TODO: Make this dynamic
                let default_width = "Updated".len();

                base.iter()
                    .map(|element| {
                        *[default_width, current.len(), *element]
                            .iter()
                            .max()
                            .unwrap_or(&default_width)
                    })
                    .collect()
            });

        let rows = data.iter().map(|row| {});

        todo!()
    }
}
