// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::file;
use logger::*;
use std::process;
use walkdir::WalkDir;

/// return a named statistic for a given interface
pub fn read_network_stat(nic: &str, stat: &str) -> Result<u64, ()> {
    let path = format!("/sys/class/net/{}/statistics/{}", nic, stat);
    file::file_as_u64(&path)
}

/// returns a `Vec` of interface names
pub fn get_network_interfaces() -> Result<Vec<String>, ()> {
    let mut result = Vec::new();
    for entry in WalkDir::new("/sys/class/net/").max_depth(1) {
        if let Ok(entry) = entry {
            if let Some(s) = entry.file_name().to_str() {
                result.push(s.to_owned());
            }
        }
    }
    if result.is_empty() {
        Err(())
    } else {
        Ok(result)
    }
}

// determine if a NIC is active based on operstate
pub fn is_nic_active(nic: &str) -> bool {
    trace!("checking state: {}", nic);
    let path = format!("/sys/class/net/{}/operstate", nic);
    let operstate = file::string_from_file(&path).unwrap();
    let operstate = operstate.trim();
    trace!("nic: {} is in state: ({})", nic, operstate);
    operstate == "up"
}

pub struct FqCodelCounters {
    dropped: u64,
    overlimits: u64,
    requeues: u64,
}

impl FqCodelCounters {
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn overlimits(&self) -> u64 {
        self.overlimits
    }

    pub fn requeues(&self) -> u64 {
        self.requeues
    }
}

impl Default for FqCodelCounters {
    fn default() -> Self {
        Self {
            dropped: 0,
            overlimits: 0,
            requeues: 0,
        }
    }
}

/// grab the `fq_codel` counters from tc
pub fn fq_codel_stats(qdisc: &str) -> Result<FqCodelCounters, String> {
    let output = process::Command::new("tc")
        .arg("-s")
        .arg("qdisc")
        .arg("show")
        .arg("dev")
        .arg(qdisc)
        .output()
        .map_err(|e| format!("Failed to execute tc: {}", e))?;
    let output =
        String::from_utf8(output.stdout).map_err(|e| format!("failed to parse stdout: {}", e))?;
    let lines: Vec<&str> = output.split('\n').collect();
    if lines.len() < 3 {
        return Err("output is shorter than expected".to_owned());
    }
    let tokens: Vec<&str> = lines[0].split_whitespace().collect();
    if tokens[1] != "fq_codel" {
        return Err(format!("qdisc type {} is unsupported", tokens[1]));
    }
    let stats: Vec<&str> = lines[1].split_whitespace().collect();

    let tmp: Vec<&str> = stats[6].split(',').collect();
    let dropped = tmp[0]
        .parse()
        .map_err(|e| format!("failed to parse dropped: {}", e))?;

    let overlimits = stats[8]
        .parse()
        .map_err(|e| format!("failed to parse overlimits: {}", e))?;

    let tmp: Vec<&str> = stats[10].split(')').collect();
    let requeues = tmp[0]
        .parse()
        .map_err(|e| format!("failed to parse requeues: {}", e))?;

    Ok(FqCodelCounters {
        dropped,
        overlimits,
        requeues,
    })
}
