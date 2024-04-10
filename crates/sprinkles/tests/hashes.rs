use std::str::FromStr;

use sprinkles::packages::reference::{self, ManifestRef};

pub struct TestHandler {
    package: reference::Package,
}

// TODO: Implement tests for entire application autoupdate

impl TestHandler {
    pub fn new(package: reference::Package) -> Self {
        Self { package }
    }

    pub fn test(self) -> anyhow::Result<()> {
        todo!("Implement tests for entire application autoupdate")
    }
}

#[ignore = "not yet implemented"]
#[test]
fn test_handlers_implemented() -> anyhow::Result<()> {
    let package = reference::Package {
        manifest: ManifestRef::from_str("extras/googlechrome")?,
        version: None,
    };
    let handler = TestHandler::new(package);

    handler.test()?;

    Ok(())
}
