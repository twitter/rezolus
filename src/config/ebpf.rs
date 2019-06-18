// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ebpf {
    #[serde(default = "default")]
    all: bool,
    #[serde(default = "default")]
    block: bool,
    #[serde(default = "default")]
    ext4: bool,
    #[serde(default = "default")]
    scheduler: bool,
    #[serde(default = "default")]
    xfs: bool,
}

impl Default for Ebpf {
    fn default() -> Ebpf {
        Ebpf {
            all: default(),
            block: default(),
            ext4: default(),
            scheduler: default(),
            xfs: default(),
        }
    }
}

fn default() -> bool {
    false
}

impl Ebpf {
    #[allow(dead_code)]
    pub fn block(&self) -> bool {
        self.all || self.block
    }

    #[allow(dead_code)]
    pub fn ext4(&self) -> bool {
        self.all || self.ext4
    }

    #[allow(dead_code)]
    pub fn scheduler(&self) -> bool {
        self.all || self.scheduler
    }

    #[allow(dead_code)]
    pub fn xfs(&self) -> bool {
        self.all || self.xfs
    }
}
