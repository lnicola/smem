use humansize::file_size_opts::{FileSizeOpts, CONVENTIONAL};
use humansize::FileSize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader};
use structopt::StructOpt;

use self::options::Options;

mod options;

struct ProcessStatistics {
    pid: u16,
    uid: i32,
    username: String,
    command: String,
    cmdline: String,
    rss: usize,
    pss: usize,
    uss: usize,
    swap: usize,
}

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
) -> Result<Option<ProcessStatistics>, io::Error> {
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
    let statistics = ProcessStatistics {
        pid,
        uid,
        username,
        command,
        cmdline,
        pss,
        rss,
        uss,
        swap,
    };
    Ok(Some(statistics))
}

type FieldPrinter = Fn(&ProcessStatistics, &Options, &FileSizeOpts);

fn print_pid(process: &ProcessStatistics, _: &Options, _: &FileSizeOpts) {
    print!("{:10} ", process.pid);
}

fn print_user(process: &ProcessStatistics, opts: &Options, _: &FileSizeOpts) {
    if opts.numeric {
        print!("{:10} ", process.uid);
    } else {
        print!("{:10} ", process.username);
    }
}

fn print_pss(process: &ProcessStatistics, options: &Options, size_opts: &FileSizeOpts) {
    if options.abbreviate {
        print!("{:>10} ", process.pss.file_size(&size_opts).unwrap());
    } else {
        print!("{:10} ", process.pss);
    }
}

fn print_rss(process: &ProcessStatistics, options: &Options, size_opts: &FileSizeOpts) {
    if options.abbreviate {
        print!("{:>10} ", process.rss.file_size(&size_opts).unwrap());
    } else {
        print!("{:10} ", process.rss);
    }
}

fn print_uss(process: &ProcessStatistics, options: &Options, size_opts: &FileSizeOpts) {
    if options.abbreviate {
        print!("{:>10} ", process.uss.file_size(&size_opts).unwrap());
    } else {
        print!("{:10} ", process.uss);
    }
}

fn print_swap(process: &ProcessStatistics, options: &Options, size_opts: &FileSizeOpts) {
    if options.abbreviate {
        print!("{:>10} ", process.swap.file_size(&size_opts).unwrap());
    } else {
        print!("{:10} ", process.swap);
    }
}

fn print_cmdline(process: &ProcessStatistics, _: &Options, _: &FileSizeOpts) {
    print!("{:10} ", process.cmdline);
}

fn main() {
    let mut field_printers: HashMap<String, Box<FieldPrinter>> = HashMap::new();
    field_printers.insert("PID".to_string(), Box::new(print_pid));
    field_printers.insert("User".to_string(), Box::new(print_user));
    field_printers.insert("PSS".to_string(), Box::new(print_pss));
    field_printers.insert("RSS".to_string(), Box::new(print_rss));
    field_printers.insert("USS".to_string(), Box::new(print_uss));
    field_printers.insert("Swap".to_string(), Box::new(print_swap));
    field_printers.insert("Command".to_string(), Box::new(print_cmdline));

    let options = Options::from_args();
    let has_custom_columns = options.columns.len() > 0;
    let mut active_field_printers = Vec::new();
    let mut custom_header = String::new();
    for c in &options.columns {
        active_field_printers.push(
            field_printers
                .get(c)
                .expect(&format!("Unknown column: {}", c)),
        );
        custom_header.push_str(&format!("{:>10} ", c));
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
        if has_custom_columns {
            println!("{}", custom_header);
        } else {
            println!(
                "{:>10} {:>10} {:>10} {:>10} {:>10} {:>10} Command",
                "PID", "User", "PSS", "RSS", "USS", "Swap"
            );
        }
    }
    if options.reverse {
        processes.sort_by_key(|p| Reverse(p.rss));
    } else {
        processes.sort_by_key(|p| p.rss);
    }
    let file_size_opts = FileSizeOpts {
        space: false,
        ..CONVENTIONAL
    };
    for process in processes {
        if has_custom_columns {
            for &printer in &active_field_printers {
                printer(&process, &options, &file_size_opts);
            }
        } else {
            print_pid(&process, &options, &file_size_opts);
            print_user(&process, &options, &file_size_opts);
            print_pss(&process, &options, &file_size_opts);
            print_rss(&process, &options, &file_size_opts);
            print_uss(&process, &options, &file_size_opts);
            print_swap(&process, &options, &file_size_opts);
            print_cmdline(&process, &options, &file_size_opts);
        }
        println!("");
    }
}
