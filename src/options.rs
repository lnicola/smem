use clap::{App, Arg};
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
        let matches = App::new(env!("CARGO_PKG_NAME"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name("no-header")
                    .short('H')
                    .long("no-header")
                    .about("Disable the header line"),
            )
            .arg(
                Arg::with_name("process-filter")
                    .short('P')
                    .long("processfilter")
                    .about("Process filter")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("user-filter")
                    .short('U')
                    .long("userfilter")
                    .about("User filter")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("numeric")
                    .short('n')
                    .long("numeric")
                    .about("Numeric output"),
            )
            .arg(
                Arg::with_name("reverse")
                    .short('r')
                    .long("reverse")
                    .about("Reverse sort"),
            )
            .arg(
                Arg::with_name("abbreviate")
                    .short('k')
                    .long("abbreviate")
                    .about("Show human-readable sizes"),
            )
            .arg(
                Arg::with_name("source")
                    .short('S')
                    .long("source")
                    .about("The path to /proc (the data source)")
                    .takes_value(true)
                    .default_value("/proc"),
            )
            .arg(
                Arg::with_name("fields")
                    .short('c')
                    .long("columns")
                    .about("Columns to show")
                    .takes_value(true)
                    .multiple(true)
                    .validator(|s| super::fields::Field::from_str(s)),
            )
            .arg(
                Arg::with_name("sort-field")
                    .short('s')
                    .long("sort")
                    .about("Column to sort on")
                    .takes_value(true)
                    .validator(|s| super::fields::Field::from_str(s)),
            )
            .arg(
                Arg::with_name("totals")
                    .short('t')
                    .long("totals")
                    .about("Show totals"),
            )
            .get_matches();
        Options {
            no_header: matches.is_present("no-header"),
            process_filter: matches.value_of("process-filter").map(|s| s.to_string()),
            user_filter: matches.value_of("user-filter").map(|s| s.to_string()),
            numeric: matches.is_present("numeric"),
            reverse: matches.is_present("reverse"),
            abbreviate: matches.is_present("abbreviate"),
            source: matches
                .value_of_os("source")
                .map(|s| PathBuf::from(s))
                .unwrap(),
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
