use humansize::file_size_opts::{FileSizeOpts, CONVENTIONAL};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use regex::Regex;

use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader};

use structopt::StructOpt;

use self::fields::{Field, FieldKind};
use self::options::Options;
use self::stats::{ProcessInfo, ProcessSizes, Size};

mod fields;

mod options;

mod stats;

fn parse_size(s: &str) -> usize {
    let s = &s[..s.len() - 3];
    let pos = s.rfind(' ').unwrap();
    let s = &s[pos + 1..];
    s.parse().unwrap_or_default()
}

fn parse_uid(s: &str) -> i32 {
    assert!(s.starts_with("Uid:"));
    s[4..]
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap_or(-1)
}

fn get_username(uid: u32) -> String {
    match users::get_user_by_uid(uid) {
        Some(user) => user.name().to_string_lossy().into_owned(),
        None => String::new(),
    }
}

fn get_statistics(
    entry: &DirEntry,
    process_filter: &Option<Regex>,
    user_filter: &Option<Regex>,
) -> Result<Option<ProcessInfo>, io::Error> {
    let metadata = entry.metadata()?;
    if !metadata.is_dir() {
        return Ok(None);
    }
    let path = entry.path();

    let pid = if let Some(pid) = path
        .file_name()
        .and_then(|dir_name| dir_name.to_str())
        .and_then(|pid| pid.parse::<u16>().ok())
    {
        pid
    } else {
        return Ok(None);
    };

    let mut uid = 0;
    let reader = BufReader::new(File::open(&path.join("status"))?);
    for line in reader.lines() {
        let line = line?;
        if line.starts_with("Uid:") {
            uid = parse_uid(&line);
            break;
        }
    }
    let username = get_username(uid as u32);

    if let Some(re) = user_filter.as_ref() {
        if !re.is_match(&username) {
            return Ok(None);
        }
    }

    let mut command = fs::read(&path.join("comm"))?;
    command.pop();
    let command = String::from_utf8_lossy(&command).into_owned();

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
    let cmdline = String::from_utf8_lossy(&cmdline).into_owned();

    if let Some(re) = process_filter.as_ref() {
        if !re.is_match(&command) && !re.is_match(&cmdline) {
            return Ok(None);
        }
    }

    let reader = BufReader::new(File::open(&path.join("smaps"))?);

    let mut pss = 0;
    let mut rss = 0;
    let mut private_clean = 0;
    let mut private_dirty = 0;
    let mut swap = 0;

    for line in reader.lines() {
        let line = line?;

        let field;
        if line.starts_with("Pss:") {
            field = &mut pss;
        } else if line.starts_with("Rss:") {
            field = &mut rss;
        } else if line.starts_with("Private_Clean:") {
            field = &mut private_clean;
        } else if line.starts_with("Private_Dirty:") {
            field = &mut private_dirty;
        } else if line.starts_with("Swap:") {
            field = &mut swap;
        } else {
            continue;
        }

        *field += parse_size(&line)
    }

    let uss = private_clean + private_dirty;
    let statistics = ProcessInfo {
        pid,
        uid,
        username,
        command,
        cmdline,
        sizes: ProcessSizes {
            pss: Size(pss),
            rss: Size(rss),
            uss: Size(uss),
            swap: Size(swap),
        },
    };
    Ok(Some(statistics))
}

fn main() {
    let default_fields = vec![
        Field::Pid,
        Field::User,
        Field::Pss,
        Field::Rss,
        Field::Uss,
        Field::Swap,
        Field::Cmdline,
    ];

    let options = Options::from_args();
    let active_fields = if options.fields.len() > 0 {
        &options.fields
    } else {
        &default_fields
    };
    let mut header = String::new();
    for c in active_fields {
        header.push_str(&format!("{:>10} ", c.name()));
    }
    let process_filter = options
        .process_filter
        .as_ref()
        .map(|r| Regex::new(r).unwrap());
    let user_filter = options.user_filter.as_ref().map(|r| Regex::new(r).unwrap());

    let entries = fs::read_dir(&options.source)
        .unwrap_or_else(|e| panic!("can't read {}: {}", options.source.display(), e))
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    let mut processes = entries
        .par_iter()
        .filter_map(|e| get_statistics(e, &process_filter, &user_filter).ok())
        .flatten()
        .collect::<Vec<_>>();
    if !options.no_header {
        println!("{}", header);
    }
    let sort_field = options.sort_field.unwrap_or(Field::Rss);
    if options.reverse {
        processes.sort_by(|p1, p2| p1.cmp_by(&sort_field, p2, &options).reverse());
    } else {
        processes.sort_by(|p1, p2| p1.cmp_by(&sort_field, p2, &options));
    }
    let file_size_opts = FileSizeOpts {
        space: false,
        ..CONVENTIONAL
    };
    let mut totals = ProcessSizes::new();
    for process in processes {
        for c in active_fields {
            print!("{} ", &process.format_field(c, &options, &file_size_opts));
        }
        println!("");
        totals += process.sizes;
    }
    if options.totals {
        println!(
            "--------------------------------------------------------------------------------"
        );
        for c in active_fields {
            if c.kind(&options) == FieldKind::Size {
                print!("{} ", totals.format_field(c, &options, &file_size_opts));
            } else {
                print!("{:10} ", " ");
            }
        }
        println!("");
    }
}
