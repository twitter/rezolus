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
pub enum ProbeType {
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
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[cfg(feature = "bpf")]
pub struct FunctionProbe {
    pub name: String,                  // name of the function to probe
    pub handler: String,               // name of the probe handler
    pub probe_type: ProbeType,         // function type, kernel, user or tracepoint.
    pub probe_location: ProbeLocation, // probe location, at entry or at return.
    pub binary_path: Option<String>,   // required for user probe only.
    pub sub_system: Option<String>,    // required for tracepoint only.
}

#[cfg(feature = "bpf")]
impl FunctionProbe {
    // try attach all probes to a bpf instance.
    #[allow(unused_assignments)]
    pub fn try_attach_to_bpf(&self, bpf: &mut bcc::BPF) -> Result<(), anyhow::Error> {
        match self.probe_type {
            ProbeType::Kernel => match self.probe_location {
                ProbeLocation::Entry => bcc::Kprobe::new()
                    .handler(self.handler.as_str())
                    .function(self.name.as_str())
                    .attach(bpf)?,
                ProbeLocation::Return => bcc::Kretprobe::new()
                    .handler(self.handler.as_str())
                    .function(self.name.as_str())
                    .attach(bpf)?,
            },
            ProbeType::User => match self.probe_location {
                ProbeLocation::Entry => bcc::Uprobe::new()
                    .handler(self.handler.as_str())
                    .binary(self.binary_path.as_ref().unwrap().as_str())
                    .symbol(self.name.as_str())
                    .attach(bpf)?,
                ProbeLocation::Return => bcc::Uretprobe::new()
                    .handler(self.handler.as_str())
                    .binary(self.binary_path.as_ref().unwrap().as_str())
                    .symbol(self.name.as_str())
                    .attach(bpf)?,
            },
            ProbeType::Tracepoint => bcc::Tracepoint::new()
                .handler(self.handler.as_str())
                .subsystem(self.sub_system.as_ref().unwrap().as_str())
                .tracepoint(self.name.as_str())
                .attach(bpf)?,
        };
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
