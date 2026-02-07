<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.23.0]

### Breaking Changes

- Update dependencies & MSRV
- Enable unstable_linux feature by default

## [0.22.0]

### Breaking changes

- Update dependencies & MSRV

## [0.21.2]

### Changes

- Removed windows on non-windows platforms
- Updated gix and fixed compatibility issues

## [0.21.1]

### Changes

- Updated dependencies

## [0.21.0]

### Breaking Changes

- String getter methods for `ParsedVersion` now return a string slice (`&str`) instead of a reference to a string (`&String`)
- Methods returning copyable types now pass ownership of the value to the caller

## [0.20.0]

### Breaking Changes

- Update MSRV
- `HashMode::from_manifest` now checks for sourceforge/fosshub hash modes after generic hash modes
  - This allows manifest authors to specify a different form of hash extraction
    that takes precedent over sourceforge/fosshub extraction

### Added

- Implemented `get` method for `AutoupdateArchitecture` to get the config for a given architecture
- Implemented `Index` trait for `AutoupdateArchitecture` to get the config for a given architecture
- Implemented `get_mut` method for `AutoupdateArchitecture` to get a mutable reference to the config for a given architecture
- Implemented `get_best` method for `AutoupdateArchitecture` to get the best available config for a given architecture

## [0.19.1]

### Fixed

- Infinite recursion in `TryFrom` trait for `VersionHandle`

## [0.19.0]

### Added

- Method to `PackageHandle` to get local version
- Constant to `ScoopContext` to check if the context requires elevation

### Breaking Changes

- Updated gix to 0.68.x and gix-object to 0.46.x
- Updated `package::Reference` name resolution to support `manifest.json` file paths

## [0.18.0]

### Added

- Better support for compiling on non-windows platforms

### Changed

- `leaf` function now returns a `&str` instead of a `String`
- Added optional name parameter to `DownloadHandle::new`

## [0.17.0]

### Fixed

- Fix `XDG_CONFIG_HOME` type

### Changed

- Minor updates for `gix` monorepo breaking changes
- Updated non-windows error message
- Dependency updates

### Breaking Changes

- Updated MSRV

## [0.16.0]

### Breaking Changes

- Default logging directory for `User` context implementation is now `%LocalAppData%/sfsu/logs` instead
  of `<sfsu app path>/logs`
- Provide more default trait methods for `ScoopContext`. This may cause different outputs for certain implementations.
- Config type is now an associated type of `ScoopContext` rather than a generic type parameter
  - This saves a lot of boilerplate, internally and externally, having generics everywhere to account for the config
    type
- Removed `v1` feature. `v1` breaking changes have been merged in this release.
- Updated git2 to v0.19
- Renamed `AliasArray` to `NestedArray`
- Renamed `TOrArrayOfTs` to `SingleOrArray`
- Renamed `Downloader` to `DownloadHandle`
- Removed `Default` implementations for `User`
- Added fallible `Result` return for `User` and `Global`
- If Scoop path cannot be found both `User` and `Global` will return an error
  - Previously `User` would panic and `Global` would return the invalid path anyway
- Removed `Uninstaller` struct
- `Uninstaller` struct has been replaced with `Installer` in the manifest
- Installer host `run` function will now remove the installer file if `keep` is not set to true
- `CreateManifest` `from_str` now takes `impl AsRef<str>` rather than `String`
- An empty hash string now deserializes as `None` rather than throwing an error
- Feature `rustls-tls` has been removed
- Feature `rustls-tls-native-roots` has been removed
- Feature `vendored-libgit2` has been removed
- Feature `vendored-openssl` has been removed
- Feature `zlib-ng` has been removed
- Feature `zlib-ng-compat` has been removed
- Feature `zlib-stock` has been removed

### Changes

- Refactor substitutions trait. It now only requires `fn substitute` rather than `fn into_substituted`
- Do not do any manifest updates if the version is the same as the current version
- Replace `parallel` feature with `rayon` feature
- Updated `derive_more` to v1.0
- Performance improvements for `Process::find_running` when using `Process::ExactExe`
- Updated quick-xml to v0.34
- Builds will fail less often by falling back to
  the default buckets if the buckets.json file fails to download

### Added

- Added `EmptyConfig` struct for when your implementation does not have a config
- `InstallerHost` struct for running installers
- Tests for `InstallerHost` and `PowershellScript`
- `from/to_object` methods for Scoop config for converting to/from serde_json objects
- Implement `FromStr` for `ScoopBranch`
- `array` module for working with nested arrays
- Implement `DoubleEndedIterator`, `FusedIterator` and `ExactSizeIterator` for `NestedIterator`
- Sha512 hash benchmark
- Added running function for package handles
- Function for finding running processes
- Process enum for handling the finding of running processes
- Implemented `Manifest` to `PackageHandle` conversion
- Downloader trait for downloading files
- Added Summary struct for determining what will run for (un)installers and scripts
- Implemented `Runner` for (un)installers
- `VersionHandle` struct for handling version directories
- Implemented `list_versions` for `PackageHandle`
- Implemented `substitute` for `InstallerHost`
- Made substitution module public

### Removed

- Removed bench files and file benchmarks
- Removed blake3 benchmarks

## [0.15.2]

### Fixed

- Enable `tokio/fs` feature under `manifest-hashes` feature
- Fix cache path extension not being renamed by url segment

## [0.15.1]

### Fixed

- Cargo packaging
- Builds without default features

### Added

- Hashing algorithm benchmarks

For older version's changelogs, see the [releases](https://github.com/winpax/sprinkles/releases) page.

[Unreleased]: https://github.com/winpax/sprinkles/compare/v0.23.0...HEAD
[0.23.0]: https://github.com/winpax/sprinkles/releases/tag/v0.23.0
[0.22.0]: https://github.com/winpax/sprinkles/releases/tag/v0.22.0
[0.21.2]: https://github.com/winpax/sprinkles/releases/tag/v0.21.2
[0.21.1]: https://github.com/winpax/sprinkles/releases/tag/v0.21.1
[0.21.0]: https://github.com/winpax/sprinkles/releases/tag/v0.21.0
[0.20.0]: https://github.com/winpax/sprinkles/releases/tag/v0.20.0
[0.19.1]: https://github.com/winpax/sprinkles/releases/tag/v0.19.1
[0.19.0]: https://github.com/winpax/sprinkles/releases/tag/v0.19.0
[0.18.0]: https://github.com/winpax/sprinkles/releases/tag/v0.18.0
[0.17.0]: https://github.com/winpax/sprinkles/releases/tag/v0.17.0
[0.16.0]: https://github.com/winpax/sprinkles/releases/tag/v0.16.0
[0.15.2]: https://github.com/winpax/sprinkles/releases/tag/v0.15.2
[0.15.1]: https://github.com/winpax/sprinkles/releases/tag/v0.15.1
