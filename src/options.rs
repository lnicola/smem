use clap::{Arg, Command};
use std::path::PathBuf;
use std::str::FromStr;

pub struct Options {
    pub no_header: bool,
    pub process_filter: Option<String>,
    pub user_filter: Option<String>,
    pub numeric: bool,
    pub reverse: bool,
    pub abbreviate: bool,
    pub source: PathBuf,
    pub fields: Vec<super::fields::Field>,
    pub sort_field: Option<super::fields::Field>,
    pub totals: bool,
}

impl Options {
    pub fn from_args() -> Options {
        let matches = Command::new(env!("CARGO_PKG_NAME"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::new("no-header")
                    .short('H')
                    .long("no-header")
                    .help("Disable the header line"),
            )
            .arg(
                Arg::new("process-filter")
                    .short('P')
                    .long("processfilter")
                    .help("Process filter")
                    .takes_value(true),
            )
            .arg(
                Arg::new("user-filter")
                    .short('U')
                    .long("userfilter")
                    .help("User filter")
                    .takes_value(true),
            )
            .arg(
                Arg::new("numeric")
                    .short('n')
                    .long("numeric")
                    .help("Numeric output"),
            )
            .arg(
                Arg::new("reverse")
                    .short('r')
                    .long("reverse")
                    .help("Reverse sort"),
            )
            .arg(
                Arg::new("abbreviate")
                    .short('k')
                    .long("abbreviate")
                    .help("Show human-readable sizes"),
            )
            .arg(
                Arg::new("source")
                    .short('S')
                    .long("source")
                    .help("The path to /proc (the data source)")
                    .takes_value(true)
                    .allow_invalid_utf8(true)
                    .default_value("/proc"),
            )
            .arg(
                Arg::new("fields")
                    .short('c')
                    .long("columns")
                    .help("Columns to show")
                    .takes_value(true)
                    .multiple_values(true)
                    .multiple_occurrences(true)
                    .validator(|s| super::fields::Field::from_str(s)),
            )
            .arg(
                Arg::new("sort-field")
                    .short('s')
                    .long("sort")
                    .help("Column to sort on")
                    .takes_value(true)
                    .validator(|s| super::fields::Field::from_str(s)),
            )
            .arg(
                Arg::new("totals")
                    .short('t')
                    .long("totals")
                    .help("Show totals"),
            )
            .get_matches();
        Options {
            no_header: matches.is_present("no-header"),
            process_filter: matches.value_of("process-filter").map(|s| s.to_string()),
            user_filter: matches.value_of("user-filter").map(|s| s.to_string()),
            numeric: matches.is_present("numeric"),
            reverse: matches.is_present("reverse"),
            abbreviate: matches.is_present("abbreviate"),
            source: matches.value_of_os("source").map(PathBuf::from).unwrap(),
            fields: matches.values_of("fields").map_or_else(Vec::new, |v| {
                v.map(|s| FromStr::from_str(s).unwrap()).collect()
            }),
            sort_field: matches
                .value_of("sort-field")
                .map(|s| FromStr::from_str(s).unwrap()),
            totals: matches.is_present("totals"),
        }
    }
}
