// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

use atomics::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ebpf {
    #[serde(default = "default")]
    all: AtomicBool,
    #[serde(default = "default")]
    block: AtomicBool,
    #[serde(default = "default")]
    ext4: AtomicBool,
    #[serde(default = "default_interval")]
    interval: AtomicOption<AtomicUsize>,
    #[serde(default = "default")]
    scheduler: AtomicBool,
    #[serde(default = "default")]
    xfs: AtomicBool,
}

impl Default for Ebpf {
    fn default() -> Ebpf {
        Ebpf {
            all: default(),
            block: default(),
            ext4: default(),
            interval: default_interval(),
            scheduler: default(),
            xfs: default(),
        }
    }
}

fn default() -> AtomicBool {
    AtomicBool::new(false)
}

fn default_interval() -> AtomicOption<AtomicUsize> {
    AtomicOption::none()
}

impl SamplerConfig for Ebpf {
    fn enabled(&self) -> bool {
        self.block() || self.ext4() || self.scheduler() || self.xfs()
    }

    fn interval(&self) -> Option<usize> {
        self.interval.load(Ordering::Relaxed)
    }
}

impl Ebpf {
    #[allow(dead_code)]
    pub fn block(&self) -> bool {
        self.all.load(Ordering::Relaxed) || self.block.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn ext4(&self) -> bool {
        self.all.load(Ordering::Relaxed) || self.ext4.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn scheduler(&self) -> bool {
        self.all.load(Ordering::Relaxed) || self.scheduler.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn xfs(&self) -> bool {
        self.all.load(Ordering::Relaxed) || self.xfs.load(Ordering::Relaxed)
    }
}
