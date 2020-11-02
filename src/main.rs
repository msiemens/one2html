#![feature(backtrace)]

use crate::cli::Opt;
use crate::utils::with_progress;
use color_eyre::eyre::Result;
use color_eyre::eyre::{eyre, ContextCompat};
use console::style;
use log::LevelFilter;
use onenote_parser::Parser;
use std::path::Path;
use std::process::exit;
use structopt::StructOpt;

mod cli;
mod notebook;
mod page;
mod section;
mod templates;
mod utils;

fn main() {
    if let Err(e) = _main() {
        eprintln!("{:?}", e);

        if let Some(bt) = e
            .downcast_ref::<onenote_parser::errors::Error>()
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
    happylog::initialize(LevelFilter::Info)?;

    let opt: Opt = Opt::from_args();

    color_eyre::install()?;

    let output_dir = opt.output;
    assert_eq!(output_dir.is_file(), false);

    for path in opt.input {
        convert(&path, &output_dir)?;
    }

    Ok(())
}

fn convert(path: &Path, output_dir: &Path) -> Result<()> {
    let mut parser = Parser::new();

    match path.extension().map(|p| p.to_string_lossy()).as_deref() {
        Some("one") => {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            println!("Processing section {}...", style(&name).bright());

            let section = with_progress("Parsing input file...", || parser.parse_section(&path))?;

            section::Renderer::new().render(&section, output_dir)?;
        }
        Some("onetoc2") => {
            let name = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();
            println!("Processing notebook {}...", style(&name).bright());

            let notebook = with_progress("[1/2] Parsing input files...", || {
                parser.parse_notebook(&path)
            })?;

            let notebook_name = path
                .parent()
                .wrap_err("Input file has no parent folder")?
                .file_name()
                .wrap_err("Parent folder has no name")?
                .to_string_lossy();

            with_progress("[2/2] Rendering sections...", || {
                notebook::Renderer::new().render(&notebook, &notebook_name, &output_dir)
            })?;
        }
        Some(ext) => return Err(eyre!("Invalid file extension: {}", ext)),
        _ => return Err(eyre!("Couldn't determine file type")),
    }

    Ok(())
}
