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
- Added sprinkles contributors to credits
- Enable `contexts` feature by default
- Logs are now moved to the new logging directory if any are found in the old location
- `app` command for managing apps

### Changed

- Moved sprinkles library to seperate repo
- Renamed sfsu-derive to sfsu-macros
- Updated sprinkles library
- Use Rust nightly toolchain
- Logs now go into `LocalAppData\sfsu\logs` instead of `<sfsu install folder>\logs`
- Run debug build on push and only run release build on release
- Internal: Do not make `wrappers` module public
- Moved `purge` command into `app` subcommand
- Internal: allow dead code in `Signature` impl (functions reserved for future use)
- Moved all app related commands into `app` subcommand, and added aliases in root command

### Removed

- `info-difftrees` feature flag
- Bot contributions from contributors list

### Fixed

- CI builds
- Re-run build.rs if executable manifest changes

For older version's changelogs, see the [releases](https://github.com/winpax/sfsu/releases) page.

[Unreleased]: https://github.com/winpax/sfsu/compare/v1.13.4...HEAD
