use os_str_bytes::OsStrBytes;
use regex::bytes::Regex;

use std::ffi::OsStr;

pub struct Filters {
    process_filter: Option<Regex>,
    user_filter: Option<Regex>,
}

impl Filters {
    pub fn new() -> Self {
        Self {
            process_filter: None,
            user_filter: None,
        }
    }

    pub fn process(&mut self, filter: &str) -> &mut Self {
        self.process_filter = Some(Regex::new(filter).unwrap());
        self
    }

    pub fn user(&mut self, filter: &str) -> &mut Self {
        self.user_filter = Some(Regex::new(filter).unwrap());
        self
    }

    pub fn accept_process(&self, s: &OsStr) -> bool {
        self.process_filter
            .as_ref()
            .map(|re| re.is_match(&s.to_bytes()))
            .unwrap_or(true)
    }

    pub fn accept_user(&self, s: &OsStr) -> bool {
        self.user_filter
            .as_ref()
            .map(|re| re.is_match(&s.to_bytes()))
            .unwrap_or(true)
    }
}
