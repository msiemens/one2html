use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "one2html")]
pub(crate) struct Opt {
    /// Input files (`.one` or `.onetoc2` files)
    #[arg(short, long, required = true, value_name = "FILE", num_args = 1..)]
    pub(crate) input: Vec<PathBuf>,

    /// Output directory
    #[arg(short, long, value_name = "DIR")]
    pub(crate) output: PathBuf,
}
