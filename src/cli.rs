use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "one2html")]
pub(crate) struct Opt {
    /// Input files (`.one` or `.onetoc2` files)
    #[structopt(short, long, required = true, parse(from_os_str))]
    pub(crate) input: Vec<PathBuf>,

    /// Output directory
    #[structopt(short, long, parse(from_os_str))]
    pub(crate) output: PathBuf,
}
