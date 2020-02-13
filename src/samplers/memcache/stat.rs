// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Statistic;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MemcacheStatistic {
    inner: String,
}

impl MemcacheStatistic {
    pub fn new(name: String) -> Self {
        Self { inner: name }
    }
}

impl Statistic for MemcacheStatistic {
    fn name(&self) -> &str {
        &self.inner
    }

    fn source(&self) -> metrics::Source {
        match self.inner.as_ref() {
            "data_read" | "data_written" | "cmd_total" | "conn_total" | "conn_yield" => {
                metrics::Source::Counter
            }
            "hotkey_bw" | "hotkey_qps" => metrics::Source::Gauge,
            _ => metrics::Source::Gauge,
        }
    }
}
