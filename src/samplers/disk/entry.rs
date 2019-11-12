// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use super::*;
use crate::common::{file, SECTOR_SIZE};
use std::iter::Sum;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Entry {
    read_bytes: u64,
    read_ops: u64,
    write_bytes: u64,
    write_ops: u64,
    discard_bytes: u64,
    discard_ops: u64,
}

impl Entry {
    pub fn discard_bytes(&self) -> u64 {
        self.discard_bytes
    }

    pub fn read_bytes(&self) -> u64 {
        self.read_bytes
    }

    pub fn write_bytes(&self) -> u64 {
        self.write_bytes
    }

    pub fn discard_ops(&self) -> u64 {
        self.discard_ops
    }

    pub fn read_ops(&self) -> u64 {
        self.read_ops
    }

    pub fn write_ops(&self) -> u64 {
        self.write_ops
    }

    pub fn for_device(device: &Device) -> Self {
        if let Ok(content) =
            file::string_from_file(format!("/sys/class/block/{}/stat", device.name().unwrap()))
        {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() < 11 {
                debug!(
                    "Unable to parse stats for block device: {}",
                    device.name().unwrap_or_else(|| "Total".to_owned())
                );
            }
            Entry {
                read_ops: parts[0].parse().unwrap_or(0),
                read_bytes: parts[2].parse().unwrap_or(0) * SECTOR_SIZE,
                write_ops: parts[4].parse().unwrap_or(0),
                write_bytes: parts[6].parse().unwrap_or(0) * SECTOR_SIZE,
                discard_ops: parts.get(11).unwrap_or(&"0").parse().unwrap_or(0),
                discard_bytes: parts.get(13).unwrap_or(&"0").parse().unwrap_or(0) * SECTOR_SIZE,
            }
        } else {
            Entry::default()
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            read_bytes: 0,
            read_ops: 0,
            write_bytes: 0,
            write_ops: 0,
            discard_bytes: 0,
            discard_ops: 0,
        }
    }
}

impl Add for Entry {
    type Output = Entry;

    fn add(self, rhs: Entry) -> Entry {
        Entry {
            read_bytes: self.read_bytes.wrapping_add(rhs.read_bytes),
            read_ops: self.read_ops.wrapping_add(rhs.read_ops),
            write_bytes: self.write_bytes.wrapping_add(rhs.write_bytes),
            write_ops: self.write_ops.wrapping_add(rhs.write_ops),
            discard_bytes: self.discard_bytes.wrapping_add(rhs.discard_bytes),
            discard_ops: self.discard_ops.wrapping_add(rhs.discard_ops),
        }
    }
}

impl<'a> Sum<&'a Entry> for Entry {
    fn sum<I: Iterator<Item = &'a Entry>>(iter: I) -> Entry {
        iter.fold(Entry::default(), Add::add)
    }
}

impl<'a> Add<&'a Entry> for Entry {
    type Output = Entry;

    fn add(self, rhs: &'a Entry) -> Entry {
        Entry {
            read_bytes: self.read_bytes.wrapping_add(rhs.read_bytes),
            read_ops: self.read_ops.wrapping_add(rhs.read_ops),
            write_bytes: self.write_bytes.wrapping_add(rhs.write_bytes),
            write_ops: self.write_ops.wrapping_add(rhs.write_ops),
            discard_bytes: self.discard_bytes.wrapping_add(rhs.discard_bytes),
            discard_ops: self.discard_ops.wrapping_add(rhs.discard_ops),
        }
    }
}

impl<'a, 'b> Add<&'b Entry> for &'a Entry {
    type Output = Entry;

    fn add(self, rhs: &'b Entry) -> Entry {
        Entry {
            read_bytes: self.read_bytes.wrapping_add(rhs.read_bytes),
            read_ops: self.read_ops.wrapping_add(rhs.read_ops),
            write_bytes: self.write_bytes.wrapping_add(rhs.write_bytes),
            write_ops: self.write_ops.wrapping_add(rhs.write_ops),
            discard_bytes: self.discard_bytes.wrapping_add(rhs.discard_bytes),
            discard_ops: self.discard_ops.wrapping_add(rhs.discard_ops),
        }
    }
}

impl Sub for Entry {
    type Output = Entry;

    fn sub(self, rhs: Entry) -> Entry {
        Entry {
            read_bytes: self.read_bytes.wrapping_sub(rhs.read_bytes),
            read_ops: self.read_ops.wrapping_sub(rhs.read_ops),
            write_bytes: self.write_bytes.wrapping_sub(rhs.write_bytes),
            write_ops: self.write_ops.wrapping_sub(rhs.write_ops),
            discard_bytes: self.discard_bytes.wrapping_sub(rhs.discard_bytes),
            discard_ops: self.discard_ops.wrapping_sub(rhs.discard_ops),
        }
    }
}
