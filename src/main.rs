#![deny(clippy::disallowed_methods, clippy::disallowed_types)]

use clap::Parser;
use color_eyre::eyre::Result;
use log::LevelFilter;
use one2html::{MathTarget, NoteTagIcons};
use onenote_parser::FileSystem;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::path::PathBuf;
use std::process::exit;
use typed_path::NativePath;

#[derive(Parser, Debug)]
#[command(name = "one2html")]
pub(crate) struct Opt {
    /// Input files (`.one`, `.onetoc2`, or `.onepkg` files)
    #[arg(short, long, required = true, value_name = "FILE", num_args = 1..)]
    pub(crate) input: Vec<PathBuf>,

    /// Output directory
    #[arg(short, long, value_name = "DIR")]
    pub(crate) output: PathBuf,

    /// Emit a per-section "Conversion Warnings" page listing non-fatal parser warnings
    #[arg(long)]
    pub(crate) warnings: bool,

    /// Emit a single combined Markdown document instead of an HTML site
    /// (text/links only, no image/attachment extraction -- meant for
    /// text-mode previewing). Pass `-o -` to print to stdout instead of
    /// writing a file.
    #[arg(long)]
    pub(crate) markdown: bool,

    /// How to render math equations
    #[arg(long, default_value = "mathml")]
    pub(crate) math_target: MathTarget,

    /// How to render OneNote note-tag icons
    #[arg(long, default_value = "svg")]
    pub(crate) note_tag_icons: NoteTagIcons,
}

#[cfg(feature = "backtrace")]
fn main() {
    if let Err(e) = _main() {
        eprintln!("{:?}", e);

        if let Some(bt) = e
            .downcast_ref::<onenote_parser::errors::Error>()
            .and_then(std::error::Error::source)
        {
            eprintln!();
            eprintln!("Caused by:");
            eprintln!("{}", bt)
        }

        exit(1);
    }
}

#[cfg(not(feature = "backtrace"))]
fn main() {
    if let Err(e) = _main() {
        eprintln!("{:?}", e);

        exit(1);
    }
}

#[allow(clippy::disallowed_methods)]
fn _main() -> Result<()> {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Warn,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])?;

    let opt: Opt = Opt::parse();

    color_eyre::install()?;

    let output_dir = opt.output;

    if opt.markdown {
        let to_stdout = output_dir.as_os_str() == "-";
        if !to_stdout {
            assert!(!output_dir.is_file());
        }

        let fs = onenote_parser::fs::native_fs::NativeFs {};

        for path in &opt.input {
            let markdown = one2html::convert_to_markdown(
                NativePath::new(path.as_os_str().as_encoded_bytes()).to_typed_path(),
                fs,
            )?;

            if to_stdout {
                print!("{}", markdown);
            } else {
                let out_dir = NativePath::new(output_dir.as_os_str().as_encoded_bytes()).to_typed_path();
                fs.make_dir(out_dir)?;

                let stem = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "output".to_string());
                let out_file = out_dir.join(format!("{}.md", stem));
                fs.write_file(out_file.to_path(), markdown.as_bytes())?;
            }
        }

        return Ok(());
    }

    assert!(!output_dir.is_file());

    for path in opt.input {
        one2html::convert(
            NativePath::new(path.as_os_str().as_encoded_bytes()).to_typed_path(),
            NativePath::new(output_dir.as_os_str().as_encoded_bytes()).to_typed_path(),
            one2html::Options {
                warnings: opt.warnings,
                math_target: opt.math_target,
                note_tag_icons: opt.note_tag_icons,
            },
            onenote_parser::fs::native_fs::NativeFs {},
        )?;
    }

    Ok(())
}
