use quick_xml::name::QName;

// This may well be "bad code"
// It's probably not very battle resistant.
// I'm honestly sick of dealing with XML
// As such I'm leaving it for the time being
// fuck xml :P

pub fn parse_xml(input: impl AsRef<str>, file_names: &[impl AsRef<str>]) -> Vec<(String, String)> {
    use quick_xml::{events::Event, reader::Reader};

    let input = input.as_ref();

    let mut reader = Reader::from_str(input);
    reader.trim_text(true);

    let mut buf = vec![];

    let mut about_tag = None;
    let mut hash_tag = None;

    let mut hashes = vec![];

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                if about_tag.is_some() {
                    if e.name() == QName(b"digest:sha256") {
                        if hash_tag.is_some() {
                            panic!("woops something wasnt cleaned up properly");
                        }

                        std::mem::swap(&mut about_tag, &mut hash_tag);
                    }
                } else {
                    for attribute in e.attributes().flatten() {
                        for file_name in file_names {
                            let file_name = file_name.as_ref().to_string();

                            if attribute.key == QName(b"rdf:about")
                                && attribute.value.as_ref() == file_name.as_bytes()
                            {
                                about_tag = Some(file_name);
                            }
                        }
                    }
                }
            }

            Ok(Event::Text(e)) => {
                if let Some(hash_file) = hash_tag {
                    hashes.push((hash_file, e.unescape().unwrap().to_string()));
                    about_tag = None;
                    hash_tag = None;
                }
            }

            _ => (),
        }
    }

    hashes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_finding_imagemagick_hashes() {
        const RDF_URL: &str = "https://download.imagemagick.org/archive/binaries/digest.rdf";

        let rdf_file = reqwest::blocking::get(RDF_URL).unwrap().text().unwrap();

        let hashes = parse_xml(
            rdf_file,
            &[
                "ImageMagick-i686-pc-cygwin.tar.gz",
                "ImageMagick-i386-pc-solaris2.11.tar.gz",
            ],
        );

        for (hash_file, hash) in hashes {
            match hash_file.as_str() {
                "ImageMagick-i686-pc-cygwin.tar.gz" => assert_eq!(
                    hash,
                    "2eb106e7eda2b2c8300a19eebbe8258ece5624305a2e6248da98cfbb9cccbd62"
                ),
                "ImageMagick-i386-pc-solaris2.11.tar.gz" => assert_eq!(
                    hash,
                    "ed3ec2340dd84c7b4015fcd773ac32ab80b5c268aff234225c23ba7a6a98f326"
                ),
                _ => unreachable!(),
            }
        }
    }
}
