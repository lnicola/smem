use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(short = "H", long = "no-header", help = "Disable the header line")]
    pub no_header: bool,
    #[structopt(short = "P", long = "processfilter", help = "Process filter")]
    pub process_filter: Option<String>,
    #[structopt(short = "U", long = "userfilter", help = "User filter")]
    pub user_filter: Option<String>,
    #[structopt(short = "n", long = "numeric", help = "Numeric output")]
    pub numeric: bool,
    #[structopt(short = "r", long = "reverse", help = "Reverse sort")]
    pub reverse: bool,
    #[structopt(short = "k", long = "abbreviate", help = "Show human-readable sizes")]
    pub abbreviate: bool,
    #[structopt(
        short = "S",
        long = "source",
        help = "The path to /proc (the data source)",
        default_value = "/proc",
        parse(from_os_str)
    )]
    pub source: PathBuf,
}
