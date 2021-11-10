// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

#[cfg(feature = "bpf")]
pub struct BPF {
    pub inner: bcc::BPF,
}

#[cfg(not(feature = "bpf"))]
pub struct BPF {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg(feature = "bpf")]
pub enum FunctionType {
    Kernel,
    User,
    Tracepoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg(feature = "bpf")]
pub enum ProbeLocation {
    Entry,
    Return,
}

// Define a probe.
#[cfg(feature = "bpf")]
pub struct Probe<Statistic> {
    pub func_name: String,              // name of the function to probe
    pub func_type: FunctionType,        // function type, kernel, user or tracepoint.
    pub handler: String,                // the handler function
    pub location: ProbeLocation,        // probe location, at entry or at return.
    pub statistics: Vec<Statistic>,     // statistics that require this probe.
    pub binary_path: Option<String>,    // required for user probe only.
    pub sub_system: Option<String>,     // required for tracepoint only.
}

// A collection of probes.
#[cfg(feature = "bpf")]
pub struct Probes<Statistic> {
    pub probes: Vec<Probe<Statistic>>,
}

#[cfg(feature = "bpf")]
impl<Statistic: std::cmp::PartialEq> Probes<Statistic> {
    pub fn new() -> Self {
        Probes { probes: Vec::new() }
    }

    pub fn add_kernel_probe(
        &mut self,
        func_name: String,
        handler: String,
        location: ProbeLocation,
        statistics: Vec<Statistic>,
    ) {
        self.probes.push(Probe {
            func_name,
            func_type: FunctionType::Kernel,
            handler,
            location,
            statistics,
            binary_path: None,
            sub_system: None,
        });
    }

    pub fn add_user_probe(
        &mut self,
        func_name: String,
        handler: String,
        location: ProbeLocation,
        binary_path: String,
        statistics: Vec<Statistic>,
    ) {
        self.probes.push(Probe {
            func_name,
            func_type: FunctionType::User,
            handler,
            location,
            statistics,
            binary_path: Some(binary_path),
            sub_system: None,
        });
    }

    pub fn add_tracepoint_probe(
        &mut self,
        func_name: String,
        handler: String,
        sub_system: String,
        statistics: Vec<Statistic>,
    ) {
        self.probes.push(Probe {
            func_name,
            func_type: FunctionType::Tracepoint,
            handler,
            location: ProbeLocation::Entry,
            statistics,
            binary_path: None,
            sub_system: Some(sub_system),
        });
    }

    // try attach all probes to a bpf instance.
    #[allow(unused_assignments)]
    pub fn try_attach_to_bpf(
        &self,
        bpf: &mut bcc::BPF,
        enabled_statistic: &[Statistic],
        fault_tolerant: Option<bool>,
    ) -> Result<(), anyhow::Error> {
        // we decide whether or not to attach a probe based on what statistics are enabled.
        for probe in &self.probes {
            // It should only be attached if at least one of statistic that requires it is enabled.
            let mut required = false;
            for stat in &probe.statistics {
                if enabled_statistic.contains(&stat) {
                    required = true;
                    break;
                }
            }

            // Keep the current behaviour consistent for now (which is always just attach)
            // as we move code of samplers batch by batch.
            // TODO: once all probes in all samplers are taken care of, activate the filtering behaviour.
            required = true;

            if required {
                let result = match probe.func_type {
                    FunctionType::Kernel => match probe.location {
                        ProbeLocation::Entry => bcc::Kprobe::new()
                            .handler(probe.handler.as_str())
                            .function(probe.func_name.as_str())
                            .attach(bpf),
                        ProbeLocation::Return => bcc::Kretprobe::new()
                            .handler(probe.handler.as_str())
                            .function(probe.func_name.as_str())
                            .attach(bpf),
                    },
                    FunctionType::User => match probe.location {
                        ProbeLocation::Entry => bcc::Uprobe::new()
                            .handler(probe.handler.as_str())
                            .binary(probe.binary_path.as_ref().unwrap().as_str())
                            .symbol(probe.func_name.as_str())
                            .attach(bpf),
                        ProbeLocation::Return => bcc::Uretprobe::new()
                            .handler(probe.handler.as_str())
                            .binary(probe.binary_path.as_ref().unwrap().as_str())
                            .symbol(probe.func_name.as_str())
                            .attach(bpf),
                    },
                    FunctionType::Tracepoint => bcc::Tracepoint::new()
                        .handler(probe.handler.as_str())
                        .subsystem(probe.sub_system.as_ref().unwrap().as_str())
                        .tracepoint(probe.func_name.as_str())
                        .attach(bpf),
                };
                // when there's an error, check fault_tolerance.
                if let Err(_) = &result {
                    if fault_tolerant.is_some() && fault_tolerant.unwrap() {
                        warn!(
                            "unable to attach probe to function {}",
                            probe.func_name.as_str()
                        );
                    } else {
                        // if fault_tolerant is not specified or it's false, return error here.
                        result?
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "bpf")]
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
#[cfg(feature = "bpf")]
pub fn symbol_lookup(name: &str) -> Option<String> {
    use std::fs::File;
    use std::io::prelude::*;
    use std::io::BufReader;

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

#[cfg(feature = "bpf")]
pub fn map_from_table(table: &mut bcc::table::Table) -> std::collections::HashMap<u64, u32> {
    use std::collections::HashMap;

    let mut current = HashMap::new();

    trace!("transferring data to userspace");
    for (id, mut entry) in table.iter().enumerate() {
        let mut key = [0; 4];
        if key.len() != entry.key.len() {
            // log and skip processing if the key length is unexpected
            debug!(
                "unexpected length of the entry's key, entry id: {} key length: {}",
                id,
                entry.key.len()
            );
            continue;
        }
        key.copy_from_slice(&entry.key);
        let key = u32::from_ne_bytes(key);

        let mut value = [0; 8];
        if value.len() != entry.value.len() {
            // log and skip processing if the value length is unexpected
            debug!(
                "unexpected length of the entry's value, entry id: {} value length: {}",
                id,
                entry.value.len()
            );
            continue;
        }
        value.copy_from_slice(&entry.value);
        let value = u64::from_ne_bytes(value);

        if let Some(key) = key_to_value(key as u64) {
            current.insert(key, value as u32);
        }

        // clear the source counter
        let _ = table.set(&mut entry.key, &mut [0_u8; 8]);
    }
    current
}

#[cfg(feature = "bpf")]
pub fn perf_table_to_map(table: &bcc::table::Table) -> std::collections::HashMap<u32, u64> {
    let mut map = std::collections::HashMap::new();

    for entry in table.iter() {
        let key = parse_u32(entry.key);
        let value = parse_u64(entry.value);

        map.insert(key, value);
    }

    map
}

#[cfg(feature = "bpf")]
pub fn bpf_hash_char_to_map(table: &bcc::table::Table) -> std::collections::HashMap<String, u64> {
    let mut map = std::collections::HashMap::new();

    for e in table.iter() {
        let key = parse_string(&e.key);
        let value = parse_u64(e.value);
        map.insert(key, value);
    }

    map
}

#[cfg(feature = "bpf")]
pub fn parse_u32(x: Vec<u8>) -> u32 {
    let mut v = [0_u8; 4];
    for (i, byte) in v.iter_mut().enumerate() {
        *byte = *x.get(i).unwrap_or(&0);
    }

    u32::from_ne_bytes(v)
}

#[cfg(feature = "bpf")]
pub fn parse_u64(x: Vec<u8>) -> u64 {
    let mut v = [0_u8; 8];
    for (i, byte) in v.iter_mut().enumerate() {
        *byte = *x.get(i).unwrap_or(&0);
    }

    u64::from_ne_bytes(v)
}

#[cfg(feature = "bpf")]
pub fn parse_string(x: &[u8]) -> String {
    match x.iter().position(|&r| r == 0) {
        Some(zero_pos) => String::from_utf8_lossy(&x[0..zero_pos]).to_string(),
        None => String::from_utf8_lossy(x).to_string(),
    }
}
