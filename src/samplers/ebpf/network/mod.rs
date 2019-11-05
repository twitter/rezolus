// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod statistics;

use self::statistics::Statistic;
use super::map_from_table;
use crate::common::*;
use crate::config::*;
use crate::samplers::{Common, Sampler};

use bcc;
use bcc::core::BPF;
use failure::*;
use logger::*;
use metrics::*;
use time;

use std::sync::Arc;

pub struct Network {
    bpf: BPF,
    common: Common,
}

impl Sampler for Network {
    fn new(
        config: Arc<Config>,
        metrics: Metrics<AtomicU32>,
    ) -> Result<Option<Box<dyn Sampler>>, Error> {
        if config.ebpf().network() {
            debug!("initializing");
            // load the code and compile
            let code = include_str!("bpf.c").to_string();
            let mut bpf = BPF::new(&code)?;

            // load + attach kprobes!
            let trace_transmit = bpf.load_tracepoint("trace_transmit")?;
            bpf.attach_tracepoint("net", "net_dev_queue", trace_transmit)?;

            let trace_receive = bpf.load_tracepoint("trace_receive")?;
            bpf.attach_tracepoint("net", "netif_rx", trace_receive)?;

            Ok(Some(Box::new(Self {
                bpf,
                common: Common::new(config, metrics),
            }) as Box<dyn Sampler>))
        } else {
            Ok(None)
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn name(&self) -> String {
        "ebpf::network".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        let time = time::precise_time_ns();
        self.register();
        for statistic in &[Statistic::ReceiveSize, Statistic::TransmitSize] {
            trace!("sampling {}", statistic);
            let mut table = self.bpf.table(&statistic.table_name());
            for (&value, &count) in &map_from_table(&mut table, BYTE) {
                self.common
                    .record_distribution(statistic, time, value, count);
            }
        }
        Ok(())
    }

    fn interval(&self) -> usize {
        self.common()
            .config()
            .ebpf()
            .interval()
            .unwrap_or_else(|| self.common().config().interval())
    }

    fn register(&mut self) {
        if !self.common.initialized() {
            trace!("register {}", self.name());
            for statistic in &[Statistic::ReceiveSize, Statistic::TransmitSize] {
                self.common
                    .register_distribution(statistic, MEGABYTE, 2, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        trace!("deregister {}", self.name());
        for statistic in &[Statistic::ReceiveSize, Statistic::TransmitSize] {
            self.common.delete_channel(statistic);
        }
        self.common.set_initialized(false);
    }
}
