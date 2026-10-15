# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-10-15

### Added

- Support for files created by the OneNote desktop application (2016, 2019,
  LTSC, …), in addition to files downloaded from OneDrive. The file format is
  detected automatically.
- Split the crate into a reusable library and an optional CLI binary (gated
  behind the `bin` feature), and add a `wasm32-unknown-unknown` build of the
  library.
- Support `.onepkg` packages in addition to `.one` sections and `.onetoc2`
  notebooks (gated behind the `onepkg` feature).
- LaTeX math rendering alongside MathML, selectable with `--math-target
  {mathml,latex}`; the matching MathJax build is loaded automatically.
- Optional per-section "⚠ Conversion Warnings" page via `--warnings`, listing
  non-fatal parser issues.
- Render math-only paragraphs as display-mode equations.
- Render arbitrarily nested section groups in the notebook table of contents.
- Optional emoji rendering for note-tag icons.
- Emit `<strong>`/`<em>` for bold and italic text runs.
- Emit `X-Original-Page-Id`, `X-Created-Time`, and `X-Updated-Time` meta tags
  per page.
- Support nested ink containers and improve handwriting import via batched ink
  rendering.
- Emit OneNote's handwriting OCR as an SVG `<title>` on each rendered ink
  block, making the recognized text available to screen readers
  ([#9](https://github.com/msiemens/one2html/issues/9)).
- Add a `--version` flag to the CLI.

### Changed

- Update `onenote_parser` to 2.0.0 (see its
  [release notes](https://github.com/msiemens/onenote.rs/releases/tag/v2.0.0)).
- Stream image and embedded-file payloads to disk instead of buffering them in
  memory.
- Sanitize output filenames for the target filesystem, so conversions are
  reproducible regardless of the host platform.
- Don't force the `native-fs` feature on library consumers.
- Fall back to sans-serif for Calibri and Calibri Light when those fonts are
  unavailable.

### Fixed

- Warn and skip nested section groups instead of aborting the conversion.
- Render tagged-list items as `<li>` and prevent list-tag styling from bleeding
  into following items.
- Stop dropping content when text runs have no formatting or when text-run
  boundaries exceed the available styles.
- Stop folding a single text run's style into the paragraph wrapper.
- Handle hyperlink URL markers split across multiple hidden runs.
- Position inline ink against the bounding-box size rather than the content
  size.
- Import PDF printouts as PNG when the embedded bytes are actually PNG.
- Use correct bound placement in MathML integrals.

### Security

- HTML-escape user-controllable strings — page titles, text runs, hyperlink
  URLs, and `style`/`href`/`src` attribute values — to prevent HTML/script
  injection from untrusted OneNote files.
- Prevent path traversal when writing images whose OneNote file extension
  contains path separators
  ([GHSA-86gx-fw5h-h2gp](https://github.com/msiemens/one2html/security/advisories/GHSA-86gx-fw5h-h2gp)).

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
