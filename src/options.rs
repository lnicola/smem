use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(
        short = "S",
        long = "source",
        help = "The path to /proc (the data source)",
        default_value = "/proc",
        parse(from_os_str)
    )]
    pub source: PathBuf,
}
