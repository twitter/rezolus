// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Statistic;
use rustcommon_metrics::*;

// #[derive(Eq, PartialEq, Hash)]
pub struct HttpStatistic {
    name: String,
    source: Source,
}

impl HttpStatistic {
    pub fn new(name: String, source: Source) -> Self {
        Self { name, source }
    }
}

impl Statistic<AtomicU64, AtomicU32> for HttpStatistic {
    fn name(&self) -> &str {
        &self.name
    }

    fn source(&self) -> Source {
        self.source
    }
}
