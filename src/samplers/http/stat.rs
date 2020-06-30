// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Statistic;

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Source {
    Counter,
    Gauge,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct HttpStatistic {
    name: String,
    source: Source,
}

impl HttpStatistic {
    pub fn new(name: String, source: Source) -> Self {
        Self { name, source }
    }
}

impl Statistic for HttpStatistic {
    fn name(&self) -> &str {
        &self.name
    }

    fn source(&self) -> rustcommon_metrics::Source {
        match self.source {
            Source::Counter => rustcommon_metrics::Source::Counter,
            Source::Gauge => rustcommon_metrics::Source::Gauge,
        }
    }
}
