#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hash {
    hash: String,
    hash_type: HashType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HashType {
    Sha512,
    Sha256,
    Sha1,
    MD5,
}

impl Hash {
    pub fn find_hash_in_rdf(url: String, basename: String) {
        todo!()
    }
}
