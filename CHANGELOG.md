# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added version-specific PyPy 3.11 and PyPy 3.12 wheels for Linux x86-64,
  Linux ARM64, Windows, and macOS.
- Added PyPy 3.11 and 3.12 to the Python bindings test matrix.
- Added release-time wheel-tag validation and installed-wheel smoke tests for
  every PyPy build.

### Changed

- Kept CPython wheels on `abi3` while making the CPython-only feature optional
  for version-specific PyPy builds.
- Raised the minimum supported Python version from 3.8 to 3.9 to match the
  upstream PyO3 compatibility revision required for PyPy 3.12.
- Updated `mdream` from 1.7.0 to 1.7.3.
- Updated package metadata and build documentation to advertise PyPy support.

[Unreleased]: https://github.com/RustedBytes/fast-h2m/compare/v0.4.4...HEAD
