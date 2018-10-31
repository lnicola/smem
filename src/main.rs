use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::ffi::{CStr, OsStr, OsString};
use std::fs::{self, DirEntry, File};
use std::io::{self, BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;

struct ProcessStatistics {
    pid: u16,
    uid: u16,
    username: String,
    cmdline: OsString,
    rss: usize,
    pss: usize,
    uss: usize,
}

fn parse_size(s: &str) -> usize {
    let s = &s[..s.len() - 3];
    let pos = s.rfind(' ').unwrap();
    let s = &s[pos + 1..];
    s.parse().unwrap_or_default()
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
            uid = line[4..]
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<u16>()
                .unwrap();
        }
    }
    let username = unsafe { CStr::from_ptr((*libc::getpwuid(uid as u32)).pw_name) }
        .to_string_lossy()
        .into_owned();

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
    };
    Ok(Some(statistics))
}

fn main() {
    let entries = fs::read_dir("/proc")
        .expect("can't read /proc")
        .map(|e| e.ok())
        .filter_map(|e| e)
        .collect::<Vec<_>>();
    let mut processes = entries
        .par_iter()
        .filter_map(|e| get_statistics(e).ok())
        .flatten()
        .collect::<Vec<_>>();
    println!(
        "{:>10} {:>10} {:>10} {:>10} {:>10} {}",
        "User", "PID", "PSS", "RSS", "USS", "Command"
    );
    processes.sort_by_key(|p| p.rss);
    for process in processes {
        println!(
            "{:10} {:10} {:10} {:10} {:10} {}",
            process.username,
            process.pid,
            process.pss,
            process.rss,
            process.uss,
            process.cmdline.to_string_lossy().as_ref(),
        );
    }
}
