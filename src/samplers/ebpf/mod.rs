// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod block;
mod ext4;
mod scheduler;
mod xfs;

use self::block::Block;
use self::ext4::Ext4;
use self::scheduler::Scheduler;
use self::xfs::Xfs;

use crate::config::Config;
use crate::sampler::{Sampler, SamplerError};

use logger::*;
use metrics::Recorder;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub struct EnhancedBerkeleyPacketFilter {
    samplers: Vec<Box<Sampler>>,
}

impl EnhancedBerkeleyPacketFilter {
    pub fn new(config: &Config) -> Self {
        debug!("initialize ebpf");
        let mut samplers = Vec::<Box<Sampler>>::new();
        if config.ebpf().block() {
            if let Ok(sampler) = Block::new() {
                samplers.push(Box::new(sampler));
            }
        }
        if config.ebpf().ext4() {
            if let Ok(sampler) = Ext4::new() {
                samplers.push(Box::new(sampler));
            }
        }
        if config.ebpf().scheduler() {
            if let Ok(sampler) = Scheduler::new() {
                samplers.push(Box::new(sampler));
            }
        }
        if config.ebpf().xfs() {
            if let Ok(sampler) = Xfs::new() {
                samplers.push(Box::new(sampler));
            }
        }
        Self { samplers }
    }
}

impl Sampler for EnhancedBerkeleyPacketFilter {
    fn name(&self) -> String {
        "ebpf".to_string()
    }

    fn sample(&mut self, recorder: &Recorder<u32>, config: &Config) -> Result<(), SamplerError> {
        trace!("sample {}", self.name());
        for sampler in &mut self.samplers {
            sampler.sample(recorder, config)?;
        }
        Ok(())
    }

    fn register(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("register {}", self.name());
        for sampler in &mut self.samplers {
            sampler.register(recorder, config);
        }
    }

    fn deregister(&mut self, recorder: &Recorder<u32>, config: &Config) {
        trace!("deregister {}", self.name());
        for sampler in &mut self.samplers {
            sampler.deregister(recorder, config);
        }
    }
}

pub fn key_to_value(index: u64) -> Option<u64> {
    let index = index;
    if index < 100 {
        Some(index)
    } else if index < 190 {
        Some((index - 90) * 10 + 9)
    } else if index < 280 {
        Some((index - 180) * 100 + 99)
    } else if index < 370 {
        Some((index - 270) * 1_000 + 999)
    } else if index < 460 {
        Some((index - 360) * 10_000 + 9999)
    } else {
        None
    }
}

// TODO: a result is probably more appropriate
pub fn symbol_lookup(name: &str) -> Option<String> {
    let symbols = File::open("/proc/kallsyms");
    if symbols.is_err() {
        return None;
    }

    let symbols = BufReader::new(symbols.unwrap());

    for line in symbols.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.get(2) == Some(&name) {
            return Some(parts[0].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_value() {
        assert_eq!(Some(0), key_to_value(0));
        assert_eq!(Some(99), key_to_value(99));
        assert_eq!(Some(109), key_to_value(100));
        assert_eq!(Some(999), key_to_value(189));
        assert_eq!(Some(1_099), key_to_value(190));
        assert_eq!(Some(9_999), key_to_value(279));
        assert_eq!(Some(10_999), key_to_value(280));
        assert_eq!(Some(99_999), key_to_value(369));
        assert_eq!(Some(109_999), key_to_value(370));
        assert_eq!(Some(999_999), key_to_value(459));
        assert_eq!(None, key_to_value(460));
    }
}
