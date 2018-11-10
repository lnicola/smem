use humansize::file_size_opts::FileSizeOpts;
use humansize::FileSize;

use std::cmp::Ordering;

use super::fields::Field;
use super::options::Options;

pub struct ProcessStatistics {
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

impl ProcessStatistics {
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
            Field::Pss => {
                if opts.abbreviate {
                    format!("{:>10}", self.pss.file_size(&size_opts).unwrap())
                } else {
                    format!("{:10}", self.pss)
                }
            }
            Field::Rss => {
                if opts.abbreviate {
                    format!("{:>10}", self.rss.file_size(&size_opts).unwrap())
                } else {
                    format!("{:10}", self.rss)
                }
            }
            Field::Uss => {
                if opts.abbreviate {
                    format!("{:>10} ", self.uss.file_size(&size_opts).unwrap())
                } else {
                    format!("{:10} ", self.uss)
                }
            }
            Field::Swap => {
                if opts.abbreviate {
                    format!("{:>10}", self.swap.file_size(&size_opts).unwrap())
                } else {
                    format!("{:10}", self.swap)
                }
            }
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
