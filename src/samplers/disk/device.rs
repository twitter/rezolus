// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Device {
    name: Option<String>,
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.name {
            None => write!(f, "total"),
            Some(ref name) => write!(f, "disk.{}", name),
        }
    }
}

impl Device {
    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }
}
