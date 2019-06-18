// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use logger::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Parse the contents of the file as `u64`. This method anticipates
/// it will be used on files as found in /proc which contain a single
/// literal value which can be represented by type `u64`
pub fn file_as_u64<T: AsRef<Path>>(path: T) -> Result<u64, ()> {
    let line = string_from_file(&path)?;
    line.parse().map_err(|e| {
        debug!(
            "could not parse file ({:?}) as u64: {}",
            path.as_ref().as_os_str(),
            e
        )
    })
}

pub fn string_from_file<T: AsRef<Path>>(path: T) -> Result<String, ()> {
    let f = File::open(&path).map_err(|e| {
        debug!(
            "failed to open file ({:?}): {}",
            path.as_ref().as_os_str(),
            e
        )
    })?;
    let mut f = BufReader::new(f);

    let mut line = String::new();
    f.read_line(&mut line)
        .map_err(|_| debug!("failed to read line"))?;
    let line = line.trim();
    Ok(line.to_owned())
}

pub fn nested_map_from_file<T: AsRef<Path>>(
    path: T,
) -> Result<HashMap<String, HashMap<String, u64>>, ()> {
    let mut ret = HashMap::<String, HashMap<String, u64>>::new();
    let f = File::open(&path).map_err(|e| {
        debug!(
            "failed to open file ({:#?}): {}",
            path.as_ref().as_os_str(),
            e
        )
    })?;
    let f = BufReader::new(f);
    let content: Vec<String> = f.lines().map(std::result::Result::unwrap).collect();
    for i in 0..(content.len() - 1) {
        if let Some(keys) = content.get(i) {
            if let Some(values) = content.get(i + 1) {
                let keys: Vec<&str> = keys.trim().split_whitespace().collect();
                let values: Vec<&str> = values.trim().split_whitespace().collect();
                if keys.len() > 2 {
                    let pkey = keys[0];
                    if !ret.contains_key(pkey) {
                        ret.insert(pkey.to_string(), Default::default());
                    }
                    let inner = ret.get_mut(&pkey.to_string()).unwrap();
                    for (i, key) in keys.iter().enumerate().skip(1) {
                        let value: u64 = values.get(i).unwrap_or(&"0").parse().unwrap_or(0);
                        inner.insert(key.to_string(), value);
                    }
                }
            }
        }
    }
    Ok(ret)
}
