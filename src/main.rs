#![feature(backtrace)]

use crate::utils::with_progress;
use color_eyre::eyre::Result;
use color_eyre::eyre::{eyre, ContextCompat};
use onenote_parser::Parser;
use std::env;
use std::path::PathBuf;
use std::process::exit;

mod notebook;
mod page;
mod section;
mod templates;
mod utils;

fn main() {
    if let Err(e) = _main() {
        eprintln!("{:?}", e);

        if let Some(bt) = e
            .downcast_ref::<onenote_parser::Error>()
            .and_then(std::error::Error::backtrace)
        {
            eprintln!();
            eprintln!("Caused by:");
            eprintln!("{}", bt)
        }

        exit(1);
    }
}

fn _main() -> Result<()> {
    color_eyre::install()?;

    let path = env::args()
        .nth(1)
        .expect("usage: parse <file> <output dir>");
    let path = PathBuf::from(path);

    let output_dir = env::args()
        .nth(2)
        .expect("usage: parse <file> <output dir>");
    let output_dir = PathBuf::from(output_dir);

    assert_eq!(output_dir.is_file(), false);

    let mut parser = Parser::new();

    match path.extension().map(|p| p.to_string_lossy()).as_deref() {
        Some("one") => {
            let section = with_progress("Parsing input file...", || parser.parse_section(&path))?;

            section::Renderer::new().render(&section, output_dir)?;
        }
        Some("onetoc2") => {
            let notebook =
                with_progress("Parsing input files...", || parser.parse_notebook(&path))?;

            let notebook_name = path
                .parent()
                .wrap_err("Input file has no parent folder")?
                .file_name()
                .wrap_err("Parent folder has no name")?
                .to_string_lossy();

            with_progress("Rendering sections...", || {
                notebook::Renderer::new().render(&notebook, &notebook_name, &output_dir)
            })?;
        }
        Some(ext) => return Err(eyre!("Invalid file extension: {}", ext)),
        _ => return Err(eyre!("Couldn't determine file type")),
    }

    Ok(())
}
