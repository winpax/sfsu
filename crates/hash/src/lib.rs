mod formats;

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
    pub fn from_rdf(
        source: impl AsRef<str>,
        file_names: &[impl AsRef<str>],
    ) -> Vec<(String, Self)> {
        formats::rdf::parse_xml(source, file_names)
            .into_iter()
            .map(|(hash_file, hash)| {
                (
                    hash_file,
                    Self {
                        hash,
                        hash_type: HashType::Sha256,
                    },
                )
            })
            .collect()
    }
}
