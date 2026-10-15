# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.2] - 2026-10-15

### Security

- Prevent path traversal when writing images whose OneNote file extension
  contains path separators — the remaining unsanitized sibling of the
  CVE-2026-22810 embedded-file fix
  ([GHSA-86gx-fw5h-h2gp](https://github.com/msiemens/one2html/security/advisories/GHSA-86gx-fw5h-h2gp)).

## [1.3.1] - 2026-05-15

### Fixed

- Prevent path traversal when writing embedded files to disk
  ([GHSA-gcmj-c9gg-9vh6](https://github.com/msiemens/one2html/security/advisories/GHSA-gcmj-c9gg-9vh6) /
  CVE-2026-22810).
- Log warnings instead of panicking on unsupported constructs in image,
  list, note tag, and rich text rendering.

### Changed

- Update `onenote_parser` to 1.1.1, which adds path-traversal hardening for
  `.onetoc2` section entries
  ([GHSA-4j5m-wc25-pvh7](https://github.com/msiemens/onenote.rs/security/advisories/GHSA-4j5m-wc25-pvh7)),
  guards against transaction-log offset under-/overflow, and avoids panics
  on malformed ink data.

## [1.3.0] - 2025-12-30

### Added

- Render math text content.

## [1.2.0] - 2025-12-28

### Changed

- Update to `onenote_parser` v1.0.0, adding non-legacy MS-ONESTORE support
  and improved error handling.
- Switch CLI argument parsing to `clap`.

### Fixed

- Correct line height calculation.
- Correct outline width calculation.
- Correct ink stroke opacity rendering.

## [1.1.2] - 2021-05-29

### Changed

- Internal: Updated dependencies.

## [1.1.0] - 2021-05-21

### Added

- Feature: Added support for ink drawings.

### Fixed

- Correctly calculate paragraph/list indentations.
- Fix the height of paragraphs.
- Don't depend on Rust's nightly `backtrace` feature (used in `onenote_parser`)
  when being compiled with `--no-default-features`.

## [1.0.0] - 2020-11-09

- First public release
