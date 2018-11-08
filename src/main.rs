use self::options::Options;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::ffi::{OsStr, OsString};
use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;
use structopt::StructOpt;

mod options;

struct ProcessStatistics {
    pid: u16,
    uid: i32,
    username: String,
    cmdline: OsString,
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

fn get_statistics(entry: &DirEntry) -> Result<Option<ProcessStatistics>, io::Error> {
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

    let mut cmdline = fs::read(&path.join("cmdline"))?;
    for c in &mut cmdline {
        if *c == b'\0' {
            *c = b' ';
        }
    }
    let cmdline = OsStr::from_bytes(&cmdline).to_os_string();
    if cmdline.is_empty() {
        return Ok(None);
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
        cmdline,
        pss,
        rss,
        uss,
        swap,
    };
    Ok(Some(statistics))
}

fn main() {
    let options = Options::from_args();
    let entries = fs::read_dir(&options.source)
        .unwrap_or_else(|e| panic!("can't read {}: {}", options.source.display(), e))
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>();
    let mut processes = entries
        .par_iter()
        .filter_map(|e| get_statistics(e).ok())
        .flatten()
        .collect::<Vec<_>>();
    if !options.no_header {
        println!(
            "{:>10} {:>10} {:>10} {:>10} {:>10} {:>10} Command",
            "User", "PID", "PSS", "RSS", "USS", "Swap"
        );
    }
    processes.sort_by_key(|p| p.rss);
    for process in processes {
        println!(
            "{:10} {:10} {:10} {:10} {:10} {:10} {}",
            process.username,
            process.pid,
            process.pss,
            process.rss,
            process.uss,
            process.swap,
            process.cmdline.to_string_lossy().as_ref(),
        );
    }
}
