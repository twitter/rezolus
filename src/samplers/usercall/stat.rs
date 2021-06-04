// Copyright 2019-2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use rustcommon_metrics::{AtomicU32, AtomicU64, Source, Statistic};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UsercallStatistic {
    stat_path: String,
}

impl UsercallStatistic {
    pub fn new(library: &str, func: &str) -> Self {
        Self {
            stat_path: format!("{}/{}", library, func),
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for UsercallStatistic {
    fn name(&self) -> &str {
        &self.stat_path
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}
