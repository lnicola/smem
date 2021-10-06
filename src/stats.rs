use humansize::file_size_opts::FileSizeOpts;
use humansize::FileSize;
use libc::{pid_t, uid_t};

use std::cmp::Ordering;
use std::ffi::OsString;
use std::io;
use std::io::Write;
use std::ops::{Add, AddAssign};

use super::error::Error;
use super::fields::Field;
use super::options::Options;

pub struct User {
    pub uid: uid_t,
    pub name: OsString,
}

impl User {
    pub fn new(uid: uid_t, name: OsString) -> Self {
        Self { uid, name }
    }
}

pub struct ProcessInfo {
    pub pid: pid_t,
    pub user: User,
    pub command: OsString,
    pub cmdline: OsString,
    pub sizes: ProcessSizes,
}

impl ProcessInfo {
    pub fn format_field<W: Write>(
        &self,
        mut writer: W,
        field: Field,
        opts: &Options,
        size_opts: &FileSizeOpts,
    ) -> io::Result<()> {
        match field {
            Field::Pid => write!(writer, "{:10}", self.pid),
            Field::User => {
                if opts.numeric {
                    write!(writer, "{:10}", self.user.uid)
                } else {
                    write!(writer, "{:10}", self.user.name.to_string_lossy())
                }
            }
            Field::Pss => self.sizes.pss.format_to(writer, opts, size_opts),
            Field::Rss => self.sizes.rss.format_to(writer, opts, size_opts),
            Field::Uss => self.sizes.uss.format_to(writer, opts, size_opts),
            Field::Swap => self.sizes.swap.format_to(writer, opts, size_opts),
            Field::Cmdline => write!(writer, "{:10}", self.cmdline.to_string_lossy()),
        }
    }

    pub fn cmp_by(&self, field: Field, other: &Self, opts: &Options) -> Ordering {
        match field {
            Field::Pid => self.pid.cmp(&other.pid),
            Field::User => {
                if opts.numeric {
                    self.user.uid.cmp(&other.user.uid)
                } else {
                    self.user.name.cmp(&other.user.name)
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
pub struct Size(usize);

// In bytes
impl Size {
    pub fn from_smap_entry(s: &str) -> Result<Self, Error> {
        assert!(&s[s.len() - 3..s.len() - 1] == "kB");
        let s = &s[..s.len() - 4];
        let pos = s.rfind(' ').ok_or(Error::ParseSize)?;
        let s = &s[pos + 1..];
        s.parse()
            .map(|n: usize| Size(n * 1024))
            .map_err(|_| Error::ParseSize)
    }

    pub fn format_to<W: Write>(
        &self,
        mut writer: W,
        opts: &Options,
        size_opts: &FileSizeOpts,
    ) -> io::Result<()> {
        if opts.abbreviate {
            write!(writer, "{:>10}", self.0.file_size(&size_opts).unwrap())
        } else {
            write!(writer, "{:10}", self.0)
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Self(Default::default())
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
    pub fn format_field<W: Write>(
        &self,
        writer: W,
        field: Field,
        opts: &Options,
        size_opts: &FileSizeOpts,
    ) -> io::Result<()> {
        match field {
            Field::Pss => self.pss.format_to(writer, opts, size_opts),
            Field::Rss => self.rss.format_to(writer, opts, size_opts),
            Field::Uss => self.uss.format_to(writer, opts, size_opts),
            Field::Swap => self.swap.format_to(writer, opts, size_opts),
            _ => panic!("Field not supported for totals: {}", field.name()),
        }
    }
}

impl Default for ProcessSizes {
    fn default() -> Self {
        Self {
            rss: Default::default(),
            pss: Default::default(),
            uss: Default::default(),
            swap: Default::default(),
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
