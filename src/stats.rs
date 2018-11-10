use humansize::file_size_opts::FileSizeOpts;
use humansize::FileSize;

use std::cmp::Ordering;
use std::ops::{Add, AddAssign};

use super::fields::Field;
use super::options::Options;

pub struct ProcessInfo {
    pub pid: u16,
    pub uid: i32,
    pub username: String,
    pub command: String,
    pub cmdline: String,
    pub sizes: ProcessSizes,
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
            Field::Pss => format_size(self.sizes.pss, &opts, &size_opts),
            Field::Rss => format_size(self.sizes.rss, &opts, &size_opts),
            Field::Uss => format_size(self.sizes.uss, &opts, &size_opts),
            Field::Swap => format_size(self.sizes.swap, &opts, &size_opts),
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
            Field::Pss => self.sizes.pss.cmp(&other.sizes.pss),
            Field::Rss => self.sizes.rss.cmp(&other.sizes.rss),
            Field::Uss => self.sizes.uss.cmp(&other.sizes.uss),
            Field::Swap => self.sizes.swap.cmp(&other.sizes.swap),
            Field::Cmdline => self.cmdline.cmp(&other.cmdline),
        }
    }
}

pub struct ProcessSizes {
    pub rss: usize,
    pub pss: usize,
    pub uss: usize,
    pub swap: usize,
}

impl ProcessSizes {
    pub fn new() -> Self {
        ProcessSizes {
            rss: 0,
            pss: 0,
            uss: 0,
            swap: 0,
        }
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

impl Add for ProcessSizes {
    type Output = ProcessSizes;

    fn add(self, other: ProcessSizes) -> ProcessSizes {
        ProcessSizes {
            rss: self.rss + other.rss,
            pss: self.pss + other.pss,
            uss: self.uss + other.uss,
            swap: self.swap + other.swap,
        }
    }
}

impl AddAssign for ProcessSizes {
    fn add_assign(&mut self, other: ProcessSizes) {
        *self = ProcessSizes {
            rss: self.rss + other.rss,
            pss: self.pss + other.pss,
            uss: self.uss + other.uss,
            swap: self.swap + other.swap,
        }
    }
}
