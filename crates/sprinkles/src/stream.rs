#![deprecated(note = "I didn't realise BufReader does the same shit")]

//! A module that provides a streaming iterator over a reader.

use std::io::Read;

/// A streaming iterator over a reader
pub struct Stream<const N: usize, R: Read> {
    total_length: u64,
    current: u64,
    reader: R,
    chunk: [u8; N],
}

impl<R: Read, const N: usize> Stream<N, R> {
    /// Create a new streaming iterator
    pub fn new(reader: R, total_length: u64) -> Self {
        Self {
            reader,
            total_length,
            current: 0,
            chunk: [0; N],
        }
    }
}

impl<R: Read, const N: usize> Iterator for Stream<N, R> {
    type Item = std::io::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Ensures that `read_exact` does not exhaust the reader, and throw an error in the final chunk
        if self.total_length - self.current > 1024 {
            if let Some(err) = self.reader.read_exact(&mut self.chunk).err() {
                return Some(Err(err));
            }

            self.current += 1024;

            return Some(Ok(Vec::from(self.chunk)));
        }

        let mut final_chunk = vec![];
        if let Some(err) = self.reader.read_to_end(&mut final_chunk).err() {
            return Some(Err(err));
        }

        Some(Ok(final_chunk))
    }
}
