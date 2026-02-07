use sprinkles::packages::{CreateManifest, models::manifest::Manifest};

#[test]
fn test_empty_hash_is_none() {
    const SFSU_MANIFEST: &str = include_str!("fixtures/sfsu.missing-hash.json");

    let manifest = Manifest::from_str(SFSU_MANIFEST).unwrap();

    let hash = manifest.architecture.unwrap().x64.unwrap().hash;

    assert_eq!(hash, None);
}
