# one2html

one2html lets you convert OneNote® files (sections or whole notebooks)
into HTML.

## Installation

Choose one of the following installation options:

1. Download from [GitHub Releases][latest-releases].

2. Install via [`cargo binstall`][cargo-binstall]:

```sh
cargo binstall one2html
```

3. Build from source:

```sh
cargo install --locked --no-default-features --features bin,onepkg one2html
```

All options require the latest stable [Rust][rust] compiler.

If you are using [a nightly compiler][rust-nightly], you can use
`cargo install --locked one2html` instead to print stack traces when errors
occur during OneNote file parsing.

## Usage

one2html converts files downloaded from OneDrive as well as files created by
the OneNote desktop application (2016, 2019, LTSC, …) — for example sections
from OneNote's local backup folder. The file format is detected automatically.

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

1. Visit https://onedrive.live.com
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

Notebooks exported as `.onepkg` packages can be converted directly as well:

```sh
one2html -i Notebook.onepkg -o ./output_dir/
```

## Disclaimer

This project is neither related to nor endorsed by Microsoft in any way. The
author does not have any affiliation with Microsoft.

[rust-nightly]: https://doc.rust-lang.org/book/appendix-07-nightly-rust.html#rustup-and-the-role-of-rust-nightly

[onedrive-cli]: https://github.com/lionello/onedrive-cli

[latest-releases]: https://github.com/msiemens/one2html/releases/latest

[rust]: https://www.rust-lang.org

[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
