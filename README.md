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

### Download OneNote files from OneDrive

OneNote files can be retrieved using one of two methods: Either by
downloading a notebook via the OneDrive web UI or by using the OneDrive
graph API.

To download OneNote notebooks via the OneDrive web UI, follow these
steps:

1. Visit https://onedrive.live.com/
2. Select the folder that contains your notebooks. Typically this is
   the _Documents_ folder.
3. Use the _Download_ button from the toolbar to download a ZIP file
   that contains all of your OneNote notebooks.

Alternatively you can download your OneNote files using the OneDrive API.
You can either use the API directly using the [Microsoft Graph Explorer](https://developer.microsoft.com/en-us/graph/graph-explorer)
or use a tool like [onedrive-cli](https://github.com/lionello/onedrive-cli)
to download a section (a single `.one` file), or a notebook (a folder that
contains a `.onetoc2` file along with other `.one` files).

### Convert OneNote files to HTML

OneNote sections are stored in `.one` files. To convert a section
to HTML run:

```sh
one2html Section.one ./output_dir/
```

OneNote notebooks are stored as folders that contain a `.onetoc2`
file along with the notebook's sections stored as `.one` files.
To convert a notebook to HTML run:

```sh
one2html 'Notebook/Open Notebook.onetoc2' ./output_dir/
```

## Limitations

Due to limitations of the [OneNote parser](https://github.com/msiemens/onenote.rs)
only files downloaded from OneDrive are supported. This means you can't
convert files created by the OneNote 2016 desktop application using
this tool.

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.
 