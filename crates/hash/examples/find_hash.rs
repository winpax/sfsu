use hash::formats::rdf::parse_xml;

fn main() {
    let rdf_file =
        reqwest::blocking::get("https://download.imagemagick.org/archive/binaries/digest.rdf")
            .unwrap()
            .text()
            .unwrap();

    parse_xml(
        rdf_file,
        &[
            "ImageMagick-i686-pc-cygwin.tar.gz",
            "ImageMagick-i386-pc-solaris2.11.tar.gz",
        ],
    );
}
