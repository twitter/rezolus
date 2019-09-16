// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub enum Statistic {
    RunqueueLatency,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RunqueueLatency => write!(f, "scheduler/runqueue_latency_ns"),
        }
    }
}

impl Statistic {
    pub fn table_name(&self) -> String {
        match self {
            Self::RunqueueLatency => "dist".to_string(),
        }
    }
}