// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::Statistic;

use rustcommon_metrics::*;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct MemcacheStatistic {
    inner: String,
}

impl MemcacheStatistic {
    pub fn new(name: String) -> Self {
        Self { inner: name }
    }

    pub fn summary_type(&self) -> Option<Source> {
        match self.inner.as_ref() {
            "data_read" | "data_written" | "cmd_total" | "conn_total" | "conn_yield"
            | "process_req" | "tcp_accept" | "tcp_recv_byte" | "tcp_send_byte" => {
                Some(Source::Counter)
            }
            "hotkey_bw" | "hotkey_qps" => Some(Source::Gauge),
            _ => None,
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for MemcacheStatistic {
    fn name(&self) -> &str {
        &self.inner
    }

    fn source(&self) -> Source {
        self.summary_type().unwrap_or(Source::Gauge)
    }
}
