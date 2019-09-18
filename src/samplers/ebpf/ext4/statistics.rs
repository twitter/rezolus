// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub enum Statistic {
    Fsync,
    Open,
    Read,
    Write,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Fsync => write!(f, "ext4/fsync"),
            Self::Open => write!(f, "ext4/open"),
            Self::Read => write!(f, "ext4/read"),
            Self::Write => write!(f, "ext4/write"),
        }
    }
}

impl Statistic {
    pub fn table_name(&self) -> String {
        match self {
            Self::Fsync => "fsync".to_string(),
            Self::Open => "open".to_string(),
            Self::Read => "read".to_string(),
            Self::Write => "write".to_string(),
        }
    }
}
