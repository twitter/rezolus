// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

pub enum Statistic {
    ReceiveSize,
    TransmitSize,
}

impl std::fmt::Display for Statistic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ReceiveSize => write!(f, "network/receive/size"),
            Self::TransmitSize => write!(f, "network/transmit/size"),
        }
    }
}

impl Statistic {
    pub fn table_name(&self) -> String {
        match self {
            Self::ReceiveSize => "rx_size".to_string(),
            Self::TransmitSize => "tx_size".to_string(),
        }
    }
}
