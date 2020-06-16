use humansize::file_size_opts::{FileSizeOpts, CONVENTIONAL};
use libc;
use os_str_bytes::OsStringBytes;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use std::ffi::OsString;
use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use self::error::Error;
use self::fields::{Field, FieldKind};
use self::filter::Filters;
use self::options::Options;
use self::stats::{ProcessInfo, ProcessSizes, Size};

mod error;
mod fields;
mod filter;
mod options;
mod stats;

fn parse_size(s: &str) -> usize {
    let s = &s[..s.len() - 4];
    let pos = s.rfind(' ').unwrap();
    let s = &s[pos + 1..];
    s.parse().unwrap_or_default()
}

fn parse_uid(s: &str) -> i32 {
    s[4..]
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap_or(-1)
}

fn get_username(uid: u32) -> OsString {
    match users::get_user_by_uid(uid) {
        Some(user) => user.name().to_os_string(),
        None => OsString::new(),
    }
}

fn open_smaps(path: &Path) -> io::Result<BufReader<File>> {
    let file = match File::open(&path.join("smaps_rollup")) {
        Ok(file) => file,
        Err(_) => File::open(&path.join("smaps"))?,
    };
    Ok(BufReader::new(file))
}

fn get_statistics(entry: &DirEntry, filters: &Filters) -> Result<Option<ProcessInfo>, Error> {
    let metadata = entry.metadata()?;
    if !metadata.is_dir() {
        return Ok(None);
    }
    let path = entry.path();

    let pid = if let Some(pid) = path
        .file_name()
        .and_then(|dir_name| dir_name.to_str())
        .and_then(|pid| pid.parse().ok())
    {
        pid
    } else {
        return Ok(None);
    };

    let mut line = String::new();
    let mut uid = 0;
    let mut reader = BufReader::new(File::open(&path.join("status"))?);
    while reader.read_line(&mut line).unwrap_or_default() > 0 {
        if line.starts_with("Uid:") {
            uid = parse_uid(&line);
            break;
        }
        line.clear();
    }
    let username = get_username(uid as u32);

    if !filters.accept_user(&username) {
        return Ok(None);
    }

    let mut command = fs::read(&path.join("comm"))?;
    command.pop();
    let command = OsString::from_bytes(&command)?;

    let mut cmdline = fs::read(&path.join("cmdline"))?;
    for c in &mut cmdline {
        if *c == b'\0' {
            *c = b' ';
        }
    }
    if cmdline.is_empty() {
        return Ok(None);
    }
    cmdline.pop();
    let cmdline = OsString::from_bytes(&cmdline)?;

    if !filters.accept_process(&command) && !filters.accept_process(&cmdline) {
        return Ok(None);
    }

    let mut reader = open_smaps(&path)?;

    let mut pss = 0;
    let mut rss = 0;
    let mut private_clean = 0;
    let mut private_dirty = 0;
    let mut swap = 0;

    while reader.read_line(&mut line).unwrap_or_default() > 0 {
        if line.starts_with("Pss:") {
            pss += parse_size(&line);
        } else if line.starts_with("Rss:") {
            rss += parse_size(&line);
        } else if line.starts_with("Private_Clean:") {
            private_clean += parse_size(&line);
        } else if line.starts_with("Private_Dirty:") {
            private_dirty += parse_size(&line);
        } else if line.starts_with("Swap:") {
            swap += parse_size(&line);
        }

        line.clear();
    }

    let uss = private_clean + private_dirty;
    let statistics = ProcessInfo {
        pid,
        uid,
        username,
        command,
        cmdline,
        sizes: ProcessSizes {
            pss: Size(pss * 1024),
            rss: Size(rss * 1024),
            uss: Size(uss * 1024),
            swap: Size(swap * 1024),
        },
    };
    Ok(Some(statistics))
}

fn print_processes(options: &Options) -> Result<(), Error> {
    let default_fields = vec![
        Field::Pid,
        Field::User,
        Field::Pss,
        Field::Rss,
        Field::Uss,
        Field::Swap,
        Field::Cmdline,
    ];

    let active_fields = if options.fields.is_empty() {
        &default_fields
    } else {
        &options.fields
    };

    let mut filters = filter::Filters::new();
    if let Some(ref process) = options.process_filter {
        filters.process(process);
    }
    if let Some(ref user) = options.user_filter {
        filters.user(user);
    }

    let entries = fs::read_dir(&options.source)
        .unwrap_or_else(|e| panic!("can't read {}: {}", options.source.display(), e))
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    let mut processes = entries
        .par_iter()
        .filter_map(|e| get_statistics(e, &filters).ok())
        .flatten()
        .collect::<Vec<_>>();

    if !options.no_header {
        for c in active_fields {
            if c.kind(&options) == FieldKind::Text {
                print!("{:<10} ", c.name());
            } else {
                print!("{:>10} ", c.name());
            }
        }
        println!();
    }
    let sort_field = options.sort_field.unwrap_or(Field::Rss);
    if options.reverse {
        processes.sort_by(|p1, p2| p1.cmp_by(sort_field, p2, &options).reverse());
    } else {
        processes.sort_by(|p1, p2| p1.cmp_by(sort_field, p2, &options));
    }
    let file_size_opts = FileSizeOpts {
        space: false,
        ..CONVENTIONAL
    };
    let mut totals = ProcessSizes::new();
    for process in processes {
        for &c in active_fields {
            process
                .format_field(io::stdout(), c, &options, &file_size_opts)
                .unwrap();
            print!(" ");
        }
        println!();
        totals += process.sizes;
    }
    if options.totals {
        println!(
            "--------------------------------------------------------------------------------"
        );
        for &c in active_fields {
            if c.kind(&options) == FieldKind::Size {
                totals
                    .format_field(io::stdout(), c, &options, &file_size_opts)
                    .unwrap();
                print!(" ");
            } else {
                print!("{:10} ", " ");
            }
        }
        println!();
    }
    Ok(())
}

fn run(options: &Options) -> Result<(), Error> {
    print_processes(&options)
}

fn disable_sigpipe_handling() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn main() {
    disable_sigpipe_handling();

    let options = Options::from_args();
    match run(&options) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
