use std::path::Path;

pub mod formats;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    hash: String,
    hash_type: HashType,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum HashType {
    Sha512,
    #[default]
    Sha256,
    Sha1,
    MD5,
}

impl Hash {
    pub fn find_hash_in_rdf(url: String, file_names: &[impl AsRef<Path>]) -> String {
        todo!()
    }
}
