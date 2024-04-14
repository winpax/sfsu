use std::error::Error;

use rayon::prelude::*;
use sprinkles::buckets::Bucket;

#[test]
fn test_parse_all_manifests() -> Result<(), Box<dyn Error>> {
    let buckets = Bucket::list_all()?;

    let manifests = buckets
        .into_par_iter()
        .flat_map(|bucket| bucket.list_packages())
        .flatten()
        .collect::<Vec<_>>();

    manifests.par_iter().for_each(|manifest| {
        assert!(!manifest.name.is_empty());
        assert!(!manifest.bucket.is_empty());
    });

    Ok(())
}
