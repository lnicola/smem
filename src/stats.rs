use humansize::file_size_opts::FileSizeOpts;
use humansize::FileSize;

use std::cmp::Ordering;

use super::fields::Field;
use super::options::Options;

pub struct ProcessInfo {
    pub pid: u16,
    pub uid: i32,
    pub username: String,
    pub command: String,
    pub cmdline: String,
    pub rss: usize,
    pub pss: usize,
    pub uss: usize,
    pub swap: usize,
}

fn format_size(size: usize, opts: &Options, size_opts: &FileSizeOpts) -> String {
    if opts.abbreviate {
        format!("{:>10}", size.file_size(&size_opts).unwrap())
    } else {
        format!("{:10}", size)
    }
}

impl ProcessInfo {
    pub fn format_field(&self, field: &Field, opts: &Options, size_opts: &FileSizeOpts) -> String {
        match field {
            Field::Pid => format!("{:10}", self.pid),
            Field::User => {
                if opts.numeric {
                    format!("{:10}", self.uid)
                } else {
                    format!("{:10}", self.username)
                }
            }
            Field::Pss => format_size(self.pss, &opts, &size_opts),
            Field::Rss => format_size(self.rss, &opts, &size_opts),
            Field::Uss => format_size(self.uss, &opts, &size_opts),
            Field::Swap => format_size(self.swap, &opts, &size_opts),
            Field::Cmdline => format!("{:10}", self.cmdline),
        }
    }

    pub fn cmp_by(&self, field: &Field, other: &Self, opts: &Options) -> Ordering {
        match field {
            Field::Pid => self.pid.cmp(&other.pid),
            Field::User => {
                if opts.numeric {
                    self.uid.cmp(&other.uid)
                } else {
                    self.username.cmp(&other.username)
                }
            }
            Field::Pss => self.pss.cmp(&other.pss),
            Field::Rss => self.rss.cmp(&other.rss),
            Field::Uss => self.uss.cmp(&other.uss),
            Field::Swap => self.swap.cmp(&other.swap),
            Field::Cmdline => self.cmdline.cmp(&other.cmdline),
        }
    }
}

pub struct ProcessStats {
    pub rss: usize,
    pub pss: usize,
    pub uss: usize,
    pub swap: usize,
}

impl ProcessStats {
    pub fn new() -> Self {
        ProcessStats {
            rss: 0,
            pss: 0,
            uss: 0,
            swap: 0,
        }
    }

    pub fn update(&mut self, info: &ProcessInfo) {
        self.rss += info.rss;
        self.pss += info.pss;
        self.uss += info.uss;
        self.swap += info.swap;
    }

    pub fn format_field(&self, field: &Field, opts: &Options, size_opts: &FileSizeOpts) -> String {
        match field {
            Field::Pss => format_size(self.pss, &opts, &size_opts),
            Field::Rss => format_size(self.rss, &opts, &size_opts),
            Field::Uss => format_size(self.uss, &opts, &size_opts),
            Field::Swap => format_size(self.swap, &opts, &size_opts),
            _ => panic!(format!("Field not supported for totals: {}", field.name())),
        }
    }
}
