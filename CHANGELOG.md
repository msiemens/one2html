# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v1.3.0 - 2025-12-30]

### Added

- Render math text content.

## [v1.2.0 - 2025-12-28]

### Changed

- Update to `onenote_parser` v1.0.0, adding non-legacy MS-ONESTORE support
  and improved error handling.
- Switch CLI argument parsing to `clap`.

### Fixed

- Correct line height calculation.
- Correct outline width calculation.
- Correct ink stroke opacity rendering.

## [v1.1.2 - 2021-05-29]

### Changed

- Internal: Updated dependencies.

## [v1.1.0 - 2021-05-21]

### Added

- Feature: Added support for ink drawings.

### Fixed

- Correctly calculate paragraph/list indentations.
- Fix the height of paragraphs.
- Don't depend on Rust's nightly `backtrace` feature (used in `onenote_parser`)
  when being compiled with `--no-default-features`.

## [1.0.0] - 2020-11-09

- First public release
