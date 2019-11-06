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

pub struct Xfs {
    bpf: BPF,
    common: Common,
}

impl Sampler for Xfs {
    fn new(
        config: Arc<Config>,
        metrics: Metrics<AtomicU32>,
    ) -> Result<Option<Box<dyn Sampler>>, Error> {
        if config.ebpf().xfs() {
            debug!("initializing");
            // load the code and compile
            let code = include_str!("bpf.c").to_string();
            let mut bpf = BPF::new(&code)?;

            // load + attach kprobes!
            let read_entry = bpf.load_kprobe("trace_entry")?;
            let write_entry = bpf.load_kprobe("trace_entry")?;
            let open_entry = bpf.load_kprobe("trace_entry")?;
            let fsync_entry = bpf.load_kprobe("trace_entry")?;
            let read_return = bpf.load_kprobe("trace_read_return")?;
            let write_return = bpf.load_kprobe("trace_write_return")?;
            let open_return = bpf.load_kprobe("trace_open_return")?;
            let fsync_return = bpf.load_kprobe("trace_fsync_return")?;

            bpf.attach_kprobe("xfs_file_read_iter", read_entry)?;
            bpf.attach_kprobe("xfs_file_write_iter", write_entry)?;
            bpf.attach_kprobe("xfs_file_open", open_entry)?;
            bpf.attach_kprobe("xfs_file_fsync", fsync_entry)?;
            bpf.attach_kretprobe("xfs_file_read_iter", read_return)?;
            bpf.attach_kretprobe("xfs_file_write_iter", write_return)?;
            bpf.attach_kretprobe("xfs_file_open", open_return)?;
            bpf.attach_kretprobe("xfs_file_fsync", fsync_return)?;

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
        "ebpf::xfs".to_string()
    }

    fn sample(&mut self) -> Result<(), ()> {
        // gather current state
        trace!("sampling {}", self.name());
        let time = time::precise_time_ns();
        self.register();
        for statistic in &[
            Statistic::Fsync,
            Statistic::Open,
            Statistic::Read,
            Statistic::Write,
        ] {
            trace!("sampling {}", statistic);
            let mut table = self.bpf.table(&statistic.table_name());
            for (&value, &count) in &map_from_table(&mut table, MICROSECOND) {
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
            for statistic in &[
                Statistic::Fsync,
                Statistic::Open,
                Statistic::Read,
                Statistic::Write,
            ] {
                self.common
                    .register_distribution(statistic, SECOND, 2, PERCENTILES);
            }
            self.common.set_initialized(true);
        }
    }

    fn deregister(&mut self) {
        trace!("deregister {}", self.name());
        for statistic in &[
            Statistic::Fsync,
            Statistic::Open,
            Statistic::Read,
            Statistic::Write,
        ] {
            self.common.delete_channel(statistic);
        }
        self.common.set_initialized(false);
    }
}
