// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

mod block;
mod ext4;
mod scheduler;
mod xfs;

pub use self::block::Block;
pub use self::ext4::Ext4;
pub use self::scheduler::Scheduler;
pub use self::xfs::Xfs;

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
