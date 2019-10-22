// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod block;
mod ext4;
mod scheduler;
mod tcp;
mod xfs;

pub use self::block::Block;
pub use self::ext4::Ext4;
pub use self::scheduler::Scheduler;
pub use self::tcp::Tcp;
pub use self::xfs::Xfs;

use bcc::table::Table;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

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

pub fn map_from_table(table: &mut Table, key_scale: u64) -> HashMap<u64, u32> {
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
            current.insert(key * key_scale, value as u32);
        }

        // clear the source counter
        let _ = table.set(&mut entry.key, &mut [0_u8; 8]);
    }
    current
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
