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
            Self::Fsync => write!(f, "xfs/fsync"),
            Self::Open => write!(f, "xfs/open"),
            Self::Read => write!(f, "xfs/read"),
            Self::Write => write!(f, "xfs/write"),
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