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

impl ProcessInfo {
    pub fn format_field(&self, field: Field, opts: &Options, size_opts: &FileSizeOpts) -> String {
        match field {
            Field::Pid => format!("{:10}", self.pid),
            Field::User => {
                if opts.numeric {
                    format!("{:10}", self.uid)
                } else {
                    format!("{:10}", self.username)
                }
            }
            Field::Pss => self.sizes.pss.format(&opts, &size_opts),
            Field::Rss => self.sizes.rss.format(&opts, &size_opts),
            Field::Uss => self.sizes.uss.format(&opts, &size_opts),
            Field::Swap => self.sizes.swap.format(&opts, &size_opts),
            Field::Cmdline => format!("{:10}", self.cmdline),
        }
    }

    pub fn cmp_by(&self, field: Field, other: &Self, opts: &Options) -> Ordering {
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

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub struct Size(pub usize);

impl Size {
    pub fn format(&self, opts: &Options, size_opts: &FileSizeOpts) -> String {
        if opts.abbreviate {
            format!("{:>10}", self.0.file_size(&size_opts).unwrap())
        } else {
            format!("{:10}", self.0)
        }
    }
}

pub struct ProcessSizes {
    pub rss: Size,
    pub pss: Size,
    pub uss: Size,
    pub swap: Size,
}

impl Add for Size {
    type Output = Size;

    fn add(self, other: Size) -> Size {
        Size(self.0 + other.0)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, other: Size) {
        self.0 += other.0;
    }
}

impl ProcessSizes {
    pub fn new() -> Self {
        ProcessSizes {
            rss: Size(0),
            pss: Size(0),
            uss: Size(0),
            swap: Size(0),
        }
    }

    pub fn format_field(&self, field: Field, opts: &Options, size_opts: &FileSizeOpts) -> String {
        match field {
            Field::Pss => self.pss.format(&opts, &size_opts),
            Field::Rss => self.rss.format(&opts, &size_opts),
            Field::Uss => self.uss.format(&opts, &size_opts),
            Field::Swap => self.swap.format(&opts, &size_opts),
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
        self.rss += other.rss;
        self.pss += other.pss;
        self.uss += other.uss;
        self.swap += other.swap;
    }
}
