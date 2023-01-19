use humansize::file_size_opts::{FileSizeOpts, CONVENTIONAL};
use libc::{self, pid_t, uid_t};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use stats::{Process, Size};
use users::User;

use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::os::unix::prelude::OsStringExt;
use std::path::Path;

use self::error::Error;
use self::fields::{Field, FieldKind};
use self::options::Options;
use self::stats::{ProcessDetails, ProcessSizes};

mod error;
mod fields;
mod filter;
mod options;
mod stats;

fn all_users() -> HashMap<uid_t, User> {
    unsafe { users::all_users() }
        .map(|u| (u.uid(), u))
        .collect()
}

fn parse_uid(s: &str) -> uid_t {
    s[4..]
        .split_whitespace()
        .next()
        .unwrap()
        .parse()
        .unwrap_or_default()
}

fn get_process_uid(path: &Path) -> Result<uid_t, Error> {
    let mut line = String::new();
    let mut reader = BufReader::new(File::open(path.join("status"))?);
    while reader.read_line(&mut line).unwrap_or_default() > 0 {
        if line.starts_with("Uid:") {
            return Ok(parse_uid(&line));
        }
        line.clear();
    }
    Err(Error::Processing(format!(
        "Could not find process UID entry for path: `{}'",
        path.display()
    )))
}

fn get_process_id(path: &Path) -> Result<pid_t, Error> {
    path.file_name()
        .and_then(|dir_name| dir_name.to_str())
        .and_then(|pid| pid.parse().ok())
        .ok_or_else(|| Error::Processing("Failed to get PID".to_owned()))
}

fn get_process_command(path: &Path) -> Result<OsString, Error> {
    let mut command = fs::read(path.join("comm"))?;
    command.pop();
    Ok(OsString::from_vec(command))
}

fn get_cmdline(path: &Path) -> Result<OsString, Error> {
    let mut cmdline = fs::read(path.join("cmdline"))?;
    for c in &mut cmdline {
        if *c == b'\0' {
            *c = b' ';
        }
    }
    if !cmdline.is_empty() {
        cmdline.pop();
    }
    Ok(OsString::from_vec(cmdline))
}

fn open_smaps(path: &Path) -> io::Result<BufReader<File>> {
    let file = match File::open(path.join("smaps_rollup")) {
        Ok(file) => file,
        Err(_) => File::open(path.join("smaps"))?,
    };
    Ok(BufReader::new(file))
}

fn get_memory_info(path: &Path) -> Result<ProcessSizes, Error> {
    let mut reader = open_smaps(path)?;
    let mut sizes: ProcessSizes = Default::default();
    let mut line = String::new();

    while reader.read_line(&mut line).unwrap_or_default() > 0 {
        if line.starts_with("Pss:") {
            sizes.pss += Size::from_smap_entry(&line)?;
        } else if line.starts_with("Rss:") {
            sizes.rss += Size::from_smap_entry(&line)?;
        } else if line.starts_with("Private_Clean:") || line.starts_with("Private_Dirty:") {
            sizes.uss += Size::from_smap_entry(&line)?;
        } else if line.starts_with("Swap:") {
            sizes.swap += Size::from_smap_entry(&line)?;
        }

        line.clear();
    }

    Ok(sizes)
}

fn get_process(path: &Path) -> Result<Process, Error> {
    Ok(Process {
        pid: get_process_id(path)?,
        uid: get_process_uid(path)?,
        command: get_process_command(path)?,
        cmdline: get_cmdline(path)?,
        procfs_path: path.to_path_buf(),
    })
}

fn get_process_details(
    process: &Process,
    users: &HashMap<uid_t, User>,
) -> Result<ProcessDetails, Error> {
    let user = users
        .get(&process.uid)
        .ok_or_else(|| Error::Processing("Could not get user name".to_owned()))?;
    let sizes = get_memory_info(&process.procfs_path)?;
    let statistics = ProcessDetails {
        process: process.clone(),
        user: user.clone(),
        sizes,
    };
    Ok(statistics)
}

fn all_processes(path: &Path) -> Vec<Process> {
    fs::read_dir(path)
        .unwrap_or_else(|e| panic!("can't read {}: {}", path.display(), e))
        .filter_map(|e| e.ok())
        .collect::<Vec<_>>()
        .par_iter()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => get_process(&e.path()).ok(),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn print_processes(process_details: Vec<ProcessDetails>, options: &Options) -> Result<(), Error> {
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

    if !options.no_header {
        for c in active_fields {
            if c.kind(options) == FieldKind::Text {
                print!("{:<10} ", c.name());
            } else {
                print!("{:>10} ", c.name());
            }
        }
        println!();
    }
    let file_size_opts = FileSizeOpts {
        space: false,
        ..CONVENTIONAL
    };
    let mut totals: ProcessSizes = Default::default();
    for details in process_details {
        for &c in active_fields {
            details
                .format_field(io::stdout(), c, options, &file_size_opts)
                .unwrap();
            print!(" ");
        }
        println!();
        totals += details.sizes;
    }
    if options.totals {
        println!(
            "--------------------------------------------------------------------------------"
        );
        for &c in active_fields {
            if c.kind(options) == FieldKind::Size {
                totals
                    .format_field(io::stdout(), c, options, &file_size_opts)
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
    let users = all_users();
    let processes = all_processes(&options.source);
    let mut filters = filter::Filters::new();
    if let Some(ref process) = options.process_filter {
        filters.process(process);
    }
    if let Some(ref user) = options.user_filter {
        filters.user(user);
    }

    let mut process_details = processes
        .par_iter()
        .filter(|p| {
            !p.cmdline.is_empty()
                && (filters.accept_process(&p.command) || filters.accept_process(&p.cmdline))
        })
        .filter_map(|p| get_process_details(p, &users).ok())
        .filter(|d| filters.accept_user(d.user.name()))
        .collect::<Vec<_>>();
    let sort_field = options.sort_field.unwrap_or(Field::Rss);
    if options.reverse {
        process_details.sort_by(|p1, p2| p1.cmp_by(sort_field, p2, options).reverse());
    } else {
        process_details.sort_by(|p1, p2| p1.cmp_by(sort_field, p2, options));
    }
    print_processes(process_details, options)
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
