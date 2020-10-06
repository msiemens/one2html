use onenote::Parser;
use std::env;
use std::error::Error;
use std::path::PathBuf;

mod notebook;
mod page;
mod section;
mod templates;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
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
            let section = parser.parse_section(&path)?;

            section::Renderer::new().render(&section, output_dir)?;
        }
        Some("onetoc2") => {
            let notebook = parser.parse_notebook(&path)?;
            let notebook_name = path
                .parent()
                .expect("input file has no parent folder")
                .file_name()
                .expect("parent folder has no name")
                .to_string_lossy();

            notebook::Renderer::new().render(&notebook, &notebook_name, output_dir)?;
        }
        _ => panic!("wrong file extension"),
    }

    Ok(())
}
