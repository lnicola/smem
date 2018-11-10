use humansize::file_size_opts::FileSizeOpts;
use humansize::FileSize;

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
}
