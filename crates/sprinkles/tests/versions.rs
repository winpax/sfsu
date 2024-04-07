use semver::Version;

#[test]
fn test_version() {
    let v1 = Version::new(1, 2, 3);
    assert_eq!(v1.to_string(), "1.2.3");

    let v3 = Version::new(3, 2, 1);
    assert_eq!(v3.to_string(), "3.2.1");

    let v2 = Version::new(2, 1, 3);

    assert!(v1 < v3);
    assert!(v1 < v2);

    assert_eq!(v1, Version::new(1, 2, 3));
}
