// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::common::net;
use std::iter::Sum;
use std::ops::{Add, Sub};

#[derive(Clone, Copy, Debug)]
pub struct Entry {
    rx_bytes: u64,
    rx_packets: u64,
    tx_bytes: u64,
    tx_packets: u64,
}

impl Entry {
    pub fn for_interface(interface: &str) -> Entry {
        Entry {
            rx_bytes: net::rx_bytes(interface).unwrap_or(0),
            rx_packets: net::rx_packets(interface).unwrap_or(0),
            tx_bytes: net::tx_bytes(interface).unwrap_or(0),
            tx_packets: net::tx_packets(interface).unwrap_or(0),
        }
    }

    pub fn rx_bytes(&self) -> u64 {
        self.rx_bytes
    }

    pub fn tx_bytes(&self) -> u64 {
        self.tx_bytes
    }

    pub fn rx_packets(&self) -> u64 {
        self.rx_packets
    }

    pub fn tx_packets(&self) -> u64 {
        self.tx_packets
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            rx_bytes: 0,
            rx_packets: 0,
            tx_bytes: 0,
            tx_packets: 0,
        }
    }
}

impl Add for Entry {
    type Output = Entry;

    fn add(self, rhs: Entry) -> Entry {
        Entry {
            rx_bytes: self.rx_bytes.wrapping_add(rhs.rx_bytes),
            rx_packets: self.rx_packets.wrapping_add(rhs.rx_packets),
            tx_bytes: self.tx_bytes.wrapping_add(rhs.tx_bytes),
            tx_packets: self.tx_packets.wrapping_add(rhs.tx_packets),
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
            rx_bytes: self.rx_bytes.wrapping_add(rhs.rx_bytes),
            rx_packets: self.rx_packets.wrapping_add(rhs.rx_packets),
            tx_bytes: self.tx_bytes.wrapping_add(rhs.tx_bytes),
            tx_packets: self.tx_packets.wrapping_add(rhs.tx_packets),
        }
    }
}

impl<'a, 'b> Add<&'b Entry> for &'a Entry {
    type Output = Entry;

    fn add(self, rhs: &'b Entry) -> Entry {
        Entry {
            rx_bytes: self.rx_bytes.wrapping_add(rhs.rx_bytes),
            rx_packets: self.rx_packets.wrapping_add(rhs.rx_packets),
            tx_bytes: self.tx_bytes.wrapping_add(rhs.tx_bytes),
            tx_packets: self.tx_packets.wrapping_add(rhs.tx_packets),
        }
    }
}

impl Sub for Entry {
    type Output = Entry;

    fn sub(self, rhs: Entry) -> Entry {
        Entry {
            rx_bytes: self.rx_bytes.wrapping_sub(rhs.rx_bytes),
            rx_packets: self.rx_packets.wrapping_sub(rhs.rx_packets),
            tx_bytes: self.tx_bytes.wrapping_sub(rhs.tx_bytes),
            tx_packets: self.tx_packets.wrapping_sub(rhs.tx_packets),
        }
    }
}
