#![deny(clippy::disallowed_methods, clippy::disallowed_types)]

use crate::utils::with_progress;
use color_eyre::eyre::{ContextCompat, Result, eyre};
use onenote_parser::{FileSystem, Parser};
use typed_path::TypedPath;

mod markdown;
mod notebook;
mod page;
mod section;
mod templates;
mod utils;

/// Conversion options.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// Emit a per-section "Conversion Warnings" page listing non-fatal parser warnings.
    pub warnings: bool,

    pub math_target: MathTarget,

    pub note_tag_icons: NoteTagIcons,
}

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "bin", derive(clap::ValueEnum))]
#[cfg_attr(feature = "bin", value(rename_all = "lower"))]
pub enum MathTarget {
    #[default]
    MathML,
    LaTeX,
}

/// How to render OneNote note-tag icons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "bin", derive(clap::ValueEnum))]
#[cfg_attr(feature = "bin", value(rename_all = "lower"))]
pub enum NoteTagIcons {
    /// Inline SVG icons (default). Sharper and platform-independent in pure HTML output.
    #[default]
    Svg,
    /// Unicode emoji. Useful when the HTML is post-processed into Markdown, since emoji
    /// survive the conversion as text whereas inline SVGs do not.
    Emoji,
}

pub fn convert(
    path: TypedPath,
    output_dir: TypedPath,
    options: Options,
    fs: impl FileSystem,
) -> Result<()> {
    let parser = Parser::new_with_fs(fs);

    match path.extension().map(String::from_utf8_lossy).as_deref() {
        Some("one") => {
            let name = path
                .file_name()
                .map(String::from_utf8_lossy)
                .unwrap_or_default();
            log::info!("Processing section {}...", name);

            let section = with_progress("Parsing input file...", || parser.parse_section(path))?;

            section::Renderer::new().render(&section, output_dir, options, fs)?;
        }
        Some("onetoc2") => {
            let name = path
                .parent()
                .and_then(|parent| {
                    parent
                        .file_name()
                        .map(|b| String::from_utf8_lossy(b).into_owned())
                })
                .unwrap_or_default();
            log::info!("Processing notebook {}...", name);

            let notebook = with_progress("[1/2] Parsing input files...", || {
                parser.parse_notebook(path)
            })?;

            let notebook_name = path
                .parent()
                .wrap_err("Input file has no parent folder")?
                .file_name()
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .wrap_err("Parent folder has no name")?;

            with_progress("[2/2] Rendering sections...", || {
                notebook::Renderer::new().render(&notebook, &notebook_name, options, output_dir, fs)
            })?;
        }
        #[cfg(feature = "onepkg")]
        Some("onepkg") => {
            let notebook_name = path
                .file_stem()
                .map(String::from_utf8_lossy)
                .wrap_err("Input file has no name")?;
            log::info!("Processing package {}...", notebook_name);

            let notebook = with_progress("[1/2] Parsing input package...", || {
                parser.parse_package(path)
            })?;

            with_progress("[2/2] Rendering sections...", || {
                notebook::Renderer::new().render(&notebook, &notebook_name, options, output_dir, fs)
            })?;
        }
        Some(ext) => return Err(eyre!("Invalid file extension: {}", ext)),
        _ => return Err(eyre!("Couldn't determine file type")),
    };

    Ok(())
}

/// Render a `.one`/`.onetoc2`/`.onepkg` input to a single combined Markdown
/// document (page order/structure preserved from the real object model,
/// content reduced to text + bold/italic/strikethrough + links -- see
/// `markdown` module docs). Intended for text-mode previewing (e.g. a
/// terminal file manager's viewer) rather than as a full-fidelity export.
pub fn convert_to_markdown(path: TypedPath, fs: impl FileSystem) -> Result<String> {
    let parser = Parser::new_with_fs(fs);

    match path.extension().map(String::from_utf8_lossy).as_deref() {
        Some("one") => {
            let section = with_progress("Parsing input file...", || parser.parse_section(path))?;
            markdown::render_section_standalone(&section)
        }
        Some("onetoc2") => {
            let notebook = with_progress("Parsing input files...", || parser.parse_notebook(path))?;

            let notebook_name = path
                .parent()
                .wrap_err("Input file has no parent folder")?
                .file_name()
                .map(|b| String::from_utf8_lossy(b).into_owned())
                .wrap_err("Parent folder has no name")?;

            markdown::render_notebook(&notebook, &notebook_name)
        }
        #[cfg(feature = "onepkg")]
        Some("onepkg") => {
            let notebook_name = path
                .file_stem()
                .map(String::from_utf8_lossy)
                .wrap_err("Input file has no name")?;

            let notebook =
                with_progress("Parsing input package...", || parser.parse_package(path))?;

            markdown::render_notebook(&notebook, &notebook_name)
        }
        Some(ext) => Err(eyre!("Invalid file extension: {}", ext)),
        _ => Err(eyre!("Couldn't determine file type")),
    }
}
