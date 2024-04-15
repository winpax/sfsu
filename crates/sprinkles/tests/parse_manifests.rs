use std::error::Error;

use rayon::prelude::*;
use sprinkles::{arch_field, buckets::Bucket};

#[test]
fn test_parse_all_manifests() -> Result<(), Box<dyn Error>> {
    // Some manifests (ahem unityhub) are broken and don't follow Scoop's spec
    // This list is used to skip those manifests
    // because go fuck yourself
    const BROKEN_MANIFESTS: &[&str] = &["unityhub"];

    let buckets = Bucket::list_all()?;

    let manifests = buckets
        .into_par_iter()
        .flat_map(|bucket| bucket.list_packages())
        .flatten()
        .collect::<Vec<_>>();

    manifests.par_iter().for_each(|manifest| {
        assert!(!manifest.name.is_empty());
        assert!(!manifest.bucket.is_empty());

        if BROKEN_MANIFESTS.contains(&manifest.name.as_str()) {
            return;
        }

        if let Some(autoupdate) = &manifest.autoupdate {
            let autoupdate_config = autoupdate
                .architecture
                .as_ref()
                .map(|arch| arch.merge_default(autoupdate.autoupdate_config.clone()))
                .unwrap_or(autoupdate.autoupdate_config.clone());

            assert!(
                autoupdate_config.url.is_some(),
                "URL is missing in package: {}",
                manifest.name
            );
        }
    });

    Ok(())
}
