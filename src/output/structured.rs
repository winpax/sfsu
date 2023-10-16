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

        let (headers, access): (Vec<String>, Vec<String>) = {
            // TODO: Do not panic here
            let first = data.first().expect("non-empty data");
            first
                .keys()
                .map(|v| {
                    let mut chars = v.chars();

                    match chars.next() {
                        // Should be unreachable
                        // TODO: Handle this case better
                        None => (String::new(), v.clone()),
                        Some(f) => (
                            f.to_uppercase().collect::<String>() + chars.as_str(),
                            v.clone(),
                        ),
                    }
                })
                .unzip()
        };

        let rows = data.iter().map(|row| {});

        todo!()
    }
}
