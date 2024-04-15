use std::str::FromStr;

use sprinkles::{hash::Hash, packages::reference};

pub struct TestHandler {
    package: reference::Package,
}

// TODO: Implement tests for entire application autoupdate

impl TestHandler {
    pub fn new(package: reference::Package) -> Self {
        Self { package }
    }

    pub fn test(self) -> anyhow::Result<()> {
        let manifest = self.package.manifest().unwrap();

        let hash = Hash::get_for_app(&manifest).unwrap();

        let actual_hash = manifest
            .architecture
            .unwrap()
            .x64
            .unwrap()
            .hash
            .unwrap()
            .to_string();

        assert_eq!(actual_hash, hash.hash());

        Ok(())
    }
}

#[test]
fn test_handlers_implemented() -> anyhow::Result<()> {
    let package = reference::Package::from_str("extras/googlechrome")?;

    let handler = TestHandler::new(package);

    handler.test()?;

    Ok(())
}

#[test]
fn test_googlechrome() -> anyhow::Result<()> {
    let package = reference::Package::from_str("extras/googlechrome")?;

    let handler = TestHandler::new(package);

    handler.test()?;

    Ok(())
}

#[test]
fn test_sfsu() -> anyhow::Result<()> {
    let package = reference::Package::from_str("extras/sfsu")?;

    let handler = TestHandler::new(package);

    handler.test()?;

    Ok(())
}

#[test]
fn test_keepass() -> anyhow::Result<()> {
    let package = reference::Package::from_str("extras/keepass")?;

    let handler = TestHandler::new(package);

    handler.test()?;

    Ok(())
}

#[test]
fn test_hwinfo() -> anyhow::Result<()> {
    let package = reference::Package::from_str("extras/hwinfo")?;

    let handler = TestHandler::new(package);

    handler.test()?;

    Ok(())
}