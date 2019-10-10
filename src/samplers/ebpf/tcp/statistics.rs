// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub enum Statistic {
    ConnectLatency,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ConnectLatency => write!(f, "network/tcp/connect/latency"),
        }
    }
}

impl Statistic {
    pub fn table_name(&self) -> String {
        match self {
            Self::ConnectLatency => "connlat".to_string(),
        }
    }
}
