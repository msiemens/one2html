# one2html

one2html lets you convert OneNote® files (sections or whole notebooks)
into HTML.

## Installation

At the moment only installation from source is supported. This
requires the latest stable [Rust](https://www.rust-lang.org/) compiler.
Once you've installed the Rust toolchain run: 

```sh
cargo install one2html
```

_Note:_ By default, One2HTML has support for printing stack traces when
errors occur during OneNote file parsing. This however requires a nightly
compiler. To compile with a stable compiler instead, add the `--no-default-features`
flag to the `cargo install` command.

## Usage

### Download OneNote files from OneDrive

OneNote files can be retrieved using one of two methods: Either by
using [onedrive-cli] or by downloading a notebook via the OneDrive web UI.
To do this first install `onedrive-cli` following its instructions. After
logging in using `onedrive-cli login`, you can download a section (a single
`.one` file), or a notebook (a folder that contains a `.onetoc2` file along
with other `.one` files):

```sh
# Download a notebook
onedrive-cli ls Documents/
onedrive-cli cp -R :/Documents/Notebook .

# Download a section
onedrive-cli cp -R :/Documents/Notebook/Section.one .
```

Alternatively, to download OneNote notebooks via the OneDrive web UI, follow
these steps:

1. Visit https://onedrive.live.com/
2. Select the folder that contains your notebooks. Typically this is
   the _Documents_ folder.
3. Use the _Download_ button from the toolbar to download a ZIP file
   that contains all of your OneNote notebooks.

### Convert OneNote files to HTML

OneNote sections are stored in `.one` files. To convert a section
to HTML run:

```sh
one2html -i Section.one -o ./output_dir/
```

OneNote notebooks are stored as folders that contain a `.onetoc2`
file along with the notebook's sections stored as `.one` files.
To convert a notebook to HTML run:

```sh
one2html -i 'Notebook/Open Notebook.onetoc2' -o ./output_dir/
```

## Limitations

- Due to limitations of the [OneNote parser](https://github.com/msiemens/onenote.rs)
  only files downloaded from OneDrive are supported. This means you can't
  convert files created by the OneNote 2016 desktop application using
  this tool.
- Ink drawings are not supported at the moment.
- Math formulas are not rendered properly at the moment.

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.

[onedrive-cli]: https://github.com/lionello/onedrive-cli