# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Purge command for removing persist folders
- Added dependabot config
- `MinInfo` struct from sprinkles library
- MIT license option in addition to Apache-2.0 license
- More detailed sprinkles version in clap output

### Changed

- Moved sprinkles library to seperate repo
- Renamed sfsu-derive to sfsu-macros
- Updated sprinkles library
- Use Rust nightly toolchain

### Removed

- `info-difftrees` feature flag

### Fixed

- CI builds
- Re-run build.rs if executable manifest changes

For older version's changelogs, see the [releases](https://github.com/winpax/sfsu/releases) page.

[Unreleased]: https://github.com/winpax/sfsu/compare/v1.13.4...HEAD
