// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::file::*;
use regex::Regex;

use std::fmt;

pub struct Version {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

pub fn get_version() -> Version {
    let info = string_from_file("/proc/version").unwrap();
    let regex = Regex::new(r"(\d+)\.(\d+)\.(\d+)").unwrap();
    let captures = regex.captures(&info).unwrap();
    let major: usize = captures.get(1).unwrap().as_str().parse().unwrap();
    let minor: usize = captures.get(2).unwrap().as_str().parse().unwrap();
    let patch: usize = captures.get(3).unwrap().as_str().parse().unwrap();
    Version {
        major,
        minor,
        patch,
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
