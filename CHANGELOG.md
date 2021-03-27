# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Feature: Added support for ink drawings.

### Fixed

- Correctly calculate paragraph/list indentations.
- Fix the height of paragraphs.
- Don't depend on Rust's nightly `backtrace` feature (used in `onenote_parser`)
  when being compiled with `--no-default-features`.

## [1.0.0] - 2020-11-09

- First public release