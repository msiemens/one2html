# one2html

one2html lets you convert OneNote® files (sections or whole notebooks)
into HTML.

## Installation

At the moment only installation from source is supported. This
requires the latest stable [Rust](https://www.rust-lang.org/) compiler.
Once you've installed the Rust toolchain run: 

```sh
cargo install --git https://github.com/msiemens/one2html
```

## Usage

- Get OneNote files or a OneNote notebook directory from OneDrive:
    - Either: Download the notebook folder from the OneDrive web UI
    - Or: Use the OneDrive API to download a single OneNote section file
- Run `one2html <input file/folder> <output folder>`

## Limitations

Due to limitations of the [OneNote parser](https://github.com/msiemens/onenote.rs)
only files downloaded from OneDrive are supported. This means you can't
convert files created by the OneNote 2016 desktop application using
this tool.

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.
 